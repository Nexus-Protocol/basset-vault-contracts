use super::sdk::Sdk;
use crate::error::ContractError;
use crate::state::load_last_rewards_claiming_height;
use crate::tests::sdk::{
    ANCHOR_MARKET_CONTRACT, ANCHOR_TOKEN, ANC_STABLE_SWAP_CONTRACT, CLAIMING_REWARDS_DELAY,
    OVER_LOAN_BALANCE_VALUE, PSI_DISTRIBUTOR_CONTRACT, PSI_STABLE_SWAP_CONTRACT, STABLE_DENOM,
};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::MOCK_CONTRACT_ADDR;
use cosmwasm_std::{to_binary, Coin, StdError, WasmMsg};
use cosmwasm_std::{CosmosMsg, Uint128};
use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;
use std::str::FromStr;
use yield_optimizer::basset_farmer::{
    ExecuteMsg as BassetFarmerExecuteMsg, YourselfMsg as BassetFarmerYourselfMsg,
};
use yield_optimizer::terraswap::{Asset, AssetInfo};
use yield_optimizer::{
    psi_distributor::{
        AnyoneMsg as PsiDistributorAnyoneMsg, ExecuteMsg as PsiDistributorExecuteMsg,
    },
    querier::AnchorMarketMsg,
    terraswap_pair::{Cw20HookMsg as TerraswapCw20HookMsg, ExecuteMsg as TerraswapExecuteMsg},
};

#[test]
fn honest_work() {
    let mut sdk = Sdk::init();

    let default_claiming_height =
        load_last_rewards_claiming_height(sdk.deps.as_ref().storage).unwrap();
    assert_eq!(default_claiming_height, 0);

    //send HonestWork message
    {
        let honest_work_response = sdk.user_send_honest_work(CLAIMING_REWARDS_DELAY).unwrap();

        assert_eq!(
            honest_work_response.messages,
            vec![
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: ANCHOR_MARKET_CONTRACT.to_string(),
                    msg: to_binary(&AnchorMarketMsg::ClaimRewards { to: None }).unwrap(),
                    send: vec![],
                }),
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: MOCK_CONTRACT_ADDR.to_string(),
                    msg: to_binary(&BassetFarmerExecuteMsg::Yourself {
                        yourself_msg: BassetFarmerYourselfMsg::SwapAnc
                    })
                    .unwrap(),
                    send: vec![],
                })
            ]
        );

        let claiming_height = load_last_rewards_claiming_height(sdk.deps.as_ref().storage).unwrap();
        assert_eq!(claiming_height, CLAIMING_REWARDS_DELAY);
    }

    let stable_coin_balance = Uint128(5_000_000);
    let anc_balance = Uint256::from(3_000u64);
    //send SwapAnc message
    {
        sdk.set_stable_balance(stable_coin_balance);
        sdk.set_anc_balance(anc_balance);
        let swap_anc_response = sdk.send_swap_anc().unwrap();
        assert_eq!(
            swap_anc_response.messages,
            vec![
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: ANCHOR_TOKEN.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Send {
                        amount: anc_balance.into(),
                        contract: ANC_STABLE_SWAP_CONTRACT.to_string(),
                        msg: to_binary(&TerraswapCw20HookMsg::Swap {
                            belief_price: None,
                            max_spread: None,
                            to: None,
                        })
                        .unwrap(),
                    })
                    .unwrap(),
                    send: vec![],
                }),
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: MOCK_CONTRACT_ADDR.to_string(),
                    msg: to_binary(&BassetFarmerExecuteMsg::Yourself {
                        yourself_msg: BassetFarmerYourselfMsg::DisributeRewards,
                    })
                    .unwrap(),
                    send: vec![],
                }),
            ]
        );
    }

    let stable_coin_balance_from_selling_anc = Uint128(1_000_000);
    //send DisributeRewards message
    {
        sdk.set_stable_balance(stable_coin_balance + stable_coin_balance_from_selling_anc);
        sdk.set_loan(Uint256::from(stable_coin_balance));
        sdk.set_aterra_balance(Uint256::from(stable_coin_balance));
        sdk.set_aterra_exchange_rate(Decimal256::from_str(OVER_LOAN_BALANCE_VALUE).unwrap());
        sdk.set_tax(0, 0);

        let distribute_rewards_response = sdk.send_distribute_rewards().unwrap();
        let swap_asset = Asset {
            info: AssetInfo::NativeToken {
                denom: STABLE_DENOM.to_string(),
            },
            amount: stable_coin_balance_from_selling_anc,
        };
        assert_eq!(
            distribute_rewards_response.messages,
            vec![
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: PSI_STABLE_SWAP_CONTRACT.to_string(),
                    msg: to_binary(&TerraswapExecuteMsg::Swap {
                        offer_asset: swap_asset,
                        max_spread: None,
                        belief_price: None,
                        to: Some(PSI_DISTRIBUTOR_CONTRACT.to_string()),
                    })
                    .unwrap(),
                    send: vec![Coin {
                        denom: STABLE_DENOM.to_string(),
                        amount: stable_coin_balance_from_selling_anc,
                    }],
                }),
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: PSI_DISTRIBUTOR_CONTRACT.to_string(),
                    msg: to_binary(&PsiDistributorExecuteMsg::Anyone {
                        anyone_msg: PsiDistributorAnyoneMsg::DistributeRewards,
                    })
                    .unwrap(),
                    send: vec![],
                }),
            ]
        );
    }

    //try to claim rewards again, but delay blocks is not achieved yet
    {
        let honest_work_response = sdk.user_send_honest_work(CLAIMING_REWARDS_DELAY * 2 - 1);
        assert!(honest_work_response.is_err());
        let error = honest_work_response.err().unwrap();
        if let ContractError::Std(StdError::GenericErr { msg }) = error {
            assert_eq!("claiming too often", msg);
        } else {
            panic!("wrong error type");
        };
    }

    //try to claim rewards again, after valid delay
    {
        let honest_work_response = sdk
            .user_send_honest_work(CLAIMING_REWARDS_DELAY * 2)
            .unwrap();

        assert_eq!(
            honest_work_response.messages,
            vec![
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: ANCHOR_MARKET_CONTRACT.to_string(),
                    msg: to_binary(&AnchorMarketMsg::ClaimRewards { to: None }).unwrap(),
                    send: vec![],
                }),
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: MOCK_CONTRACT_ADDR.to_string(),
                    msg: to_binary(&BassetFarmerExecuteMsg::Yourself {
                        yourself_msg: BassetFarmerYourselfMsg::SwapAnc
                    })
                    .unwrap(),
                    send: vec![],
                })
            ]
        );

        let claiming_height = load_last_rewards_claiming_height(sdk.deps.as_ref().storage).unwrap();
        assert_eq!(claiming_height, CLAIMING_REWARDS_DELAY * 2);
    }
}
