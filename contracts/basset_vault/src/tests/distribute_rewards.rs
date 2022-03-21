use super::sdk::Sdk;
use crate::tests::sdk::{
    ANCHOR_MARKET_CONTRACT, ANCHOR_TOKEN, ANC_STABLE_SWAP_CONTRACT, OVER_LOAN_BALANCE_VALUE,
    PSI_DISTRIBUTOR_CONTRACT, PSI_STABLE_SWAP_CONTRACT, STABLE_DENOM,
};
use crate::MIN_ANC_REWARDS_TO_CLAIM;
use basset_vault::basset_vault::{
    ExecuteMsg as BassetFarmerExecuteMsg, YourselfMsg as BassetFarmerYourselfMsg,
};
use basset_vault::terraswap::{Asset, AssetInfo};
use basset_vault::{
    astroport_pair::{Cw20HookMsg as AstroportCw20HookMsg, ExecuteMsg as AstroportExecuteMsg},
    psi_distributor::{
        AnyoneMsg as PsiDistributorAnyoneMsg, ExecuteMsg as PsiDistributorExecuteMsg,
    },
    querier::AnchorMarketMsg,
};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::MOCK_CONTRACT_ADDR;
use cosmwasm_std::{to_binary, Coin, Response, SubMsg, WasmMsg};
use cosmwasm_std::{CosmosMsg, Uint128};
use cw20::Cw20ExecuteMsg;
use std::str::FromStr;

#[test]
fn honest_work() {
    let mut sdk = Sdk::init();

    //NOT ENOUGHT REWARDS
    sdk.set_anc_pending_rewards(Decimal256::from_uint256(50u64));
    //send HonestWork message
    {
        let honest_work_response = sdk.user_send_honest_work().unwrap();

        assert_eq!(honest_work_response, Response::default());
    }

    //NOT ENOUGHT REWARDS
    sdk.set_anc_pending_rewards(Decimal256::from_uint256(MIN_ANC_REWARDS_TO_CLAIM));
    //send HonestWork message
    {
        let honest_work_response = sdk.user_send_honest_work().unwrap();

        assert_eq!(
            honest_work_response.messages,
            vec![
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: ANCHOR_MARKET_CONTRACT.to_string(),
                    msg: to_binary(&AnchorMarketMsg::ClaimRewards { to: None }).unwrap(),
                    funds: vec![],
                })),
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: MOCK_CONTRACT_ADDR.to_string(),
                    msg: to_binary(&BassetFarmerExecuteMsg::Yourself {
                        yourself_msg: BassetFarmerYourselfMsg::SwapAnc {}
                    })
                    .unwrap(),
                    funds: vec![],
                }))
            ]
        );
    }

    let stable_coin_balance = Uint128::new(5_000_000);
    let anc_balance = Uint256::from(3_000u64);
    //send SwapAnc message
    {
        sdk.set_stable_balance(stable_coin_balance);
        sdk.set_anc_balance(anc_balance);
        let swap_anc_response = sdk.send_swap_anc().unwrap();
        assert_eq!(
            swap_anc_response.messages,
            vec![
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: ANCHOR_TOKEN.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Send {
                        amount: anc_balance.into(),
                        contract: ANC_STABLE_SWAP_CONTRACT.to_string(),
                        msg: to_binary(&AstroportCw20HookMsg::Swap {
                            belief_price: None,
                            max_spread: None,
                            to: None,
                        })
                        .unwrap(),
                    })
                    .unwrap(),
                    funds: vec![],
                })),
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: MOCK_CONTRACT_ADDR.to_string(),
                    msg: to_binary(&BassetFarmerExecuteMsg::Yourself {
                        yourself_msg: BassetFarmerYourselfMsg::DisributeRewards {},
                    })
                    .unwrap(),
                    funds: vec![],
                })),
            ]
        );
    }

    let stable_coin_balance_from_selling_anc = Uint128::new(1_000_000);
    //send DisributeRewards message
    {
        let over_loan_balance_value = Decimal256::from_str(OVER_LOAN_BALANCE_VALUE).unwrap();
        sdk.set_stable_balance(stable_coin_balance + stable_coin_balance_from_selling_anc);
        sdk.set_loan(Uint256::from(stable_coin_balance));
        sdk.set_aterra_balance(Uint256::from(stable_coin_balance));
        sdk.set_aterra_exchange_rate(over_loan_balance_value);
        sdk.set_tax(Decimal256::zero().into(), 0);

        let aim_stable_balance: Uint256 =
            Uint256::from(stable_coin_balance) * over_loan_balance_value;
        let total_ust_value: Uint256 = Uint256::from(stable_coin_balance) * over_loan_balance_value
            + stable_coin_balance.into()
            + stable_coin_balance_from_selling_anc.into();
        let expected_rewards = total_ust_value - aim_stable_balance;
        let distribute_rewards_response = sdk.send_distribute_rewards().unwrap();
        let swap_asset = Asset {
            info: AssetInfo::NativeToken {
                denom: STABLE_DENOM.to_string(),
            },
            amount: expected_rewards.into(),
        };
        assert_eq!(
            distribute_rewards_response.messages,
            vec![
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: PSI_STABLE_SWAP_CONTRACT.to_string(),
                    msg: to_binary(&AstroportExecuteMsg::Swap {
                        offer_asset: swap_asset,
                        max_spread: None,
                        belief_price: None,
                        to: Some(PSI_DISTRIBUTOR_CONTRACT.to_string()),
                    })
                    .unwrap(),
                    funds: vec![Coin {
                        denom: STABLE_DENOM.to_string(),
                        amount: expected_rewards.into(),
                    }],
                })),
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: PSI_DISTRIBUTOR_CONTRACT.to_string(),
                    msg: to_binary(&PsiDistributorExecuteMsg::Anyone {
                        anyone_msg: PsiDistributorAnyoneMsg::DistributeRewards {},
                    })
                    .unwrap(),
                    funds: vec![],
                })),
            ]
        );
    }
}
