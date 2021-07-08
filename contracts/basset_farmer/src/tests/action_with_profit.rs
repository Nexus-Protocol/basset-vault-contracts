use crate::{
    state::{load_repaying_loan_state, store_config, RepayingLoanState},
    utils::{ActionWithProfit, RepayLoanAction},
    SubmsgIds, TOO_HIGH_BORROW_DEMAND_ERR_MSG,
};

use super::sdk::Sdk;
use crate::{
    state::{load_child_contracts_info, load_config, ChildContractsInfo, Config},
    tests::sdk::{
        ANCHOR_CUSTODY_BASSET_CONTRACT, ANCHOR_MARKET_CONTRACT, ANCHOR_OVERSEER_CONTRACT,
        ANCHOR_TOKEN, ANC_STABLE_SWAP_CONTRACT, ATERRA_TOKEN, BASSET_FARMER_CONFIG_CONTRACT,
        BASSET_TOKEN_ADDR, CLAIMING_REWARDS_DELAY, COLLATERAL_TOKEN_SYMBOL, GOVERNANCE_CONTRACT,
        NASSET_TOKEN_ADDR, NASSET_TOKEN_CODE_ID, NASSET_TOKEN_CONFIG_HOLDER_CODE_ID,
        NASSET_TOKEN_CONFIG_HOLDER_CONTRACT, NASSET_TOKEN_REWARDS_CODE_ID,
        NASSET_TOKEN_REWARDS_CONTRACT, OVER_LOAN_BALANCE_VALUE, PSI_DISTRIBUTOR_CODE_ID,
        PSI_DISTRIBUTOR_CONTRACT, PSI_STABLE_SWAP_CONTRACT, PSI_TOKEN, STABLE_DENOM,
    },
};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    attr,
    CosmosMsg,
};
use cosmwasm_std::{
    to_binary, Addr, Coin, ContractResult, Decimal, Reply, ReplyOn, Response, SubMsg,
    SubcallResponse, Uint128, WasmMsg,
};


use yield_optimizer::{
    basset_farmer::{AnyoneMsg, ExecuteMsg},
    basset_farmer_config::BorrowerActionResponse,
    psi_distributor::{
        AnyoneMsg as PsiDistributorAnyoneMsg, ExecuteMsg as PsiDistributorExecuteMsg,
    },
    querier::{
        AnchorMarketCw20Msg, AnchorMarketEpochStateResponse, AnchorMarketMsg, AnchorOverseerMsg,
        BorrowerInfoResponse,
    },
    terraswap::{Asset, AssetInfo},
    terraswap_pair::ExecuteMsg as TerraswapExecuteMsg,
    TaxInfo,
};

#[test]
fn action_with_profit_nothing() {
    let sdk = Sdk::init();
    let config = load_config(sdk.deps.as_ref().storage).unwrap();
    let tax_info = TaxInfo {
        rate: Decimal256::zero(),
        cap: Uint256::zero(),
    };

    let action_with_profit = ActionWithProfit::Nothing;
    let response = action_with_profit.to_response(&config, &tax_info).unwrap();
    assert_eq!(
        response,
        Response {
            messages: vec![],
            submessages: vec![],
            attributes: vec![
                attr("action", "distribute_rewards"),
                attr("rewards_profit", "zero"),
            ],
            data: None,
        }
    );
}

#[test]
fn action_with_profit_buy_psi() {
    let sdk = Sdk::init();
    let config = load_config(sdk.deps.as_ref().storage).unwrap();
    let tax_info = TaxInfo {
        rate: Decimal256::zero(),
        cap: Uint256::zero(),
    };

    let buy_psi_amount = Uint256::from(2_000u64);
    let action_with_profit = ActionWithProfit::BuyPsi {
        amount: buy_psi_amount,
    };
    let response = action_with_profit.to_response(&config, &tax_info).unwrap();

    let swap_asset = Asset {
        info: AssetInfo::NativeToken {
            denom: STABLE_DENOM.to_string(),
        },
        amount: tax_info.subtract_tax(buy_psi_amount).into(),
    };
    let expected_response = Response {
        messages: vec![
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
                    amount: buy_psi_amount.into(),
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
        ],
        submessages: vec![],
        attributes: vec![
            attr("action", "distribute_rewards"),
            attr("bying_psi", buy_psi_amount),
        ],
        data: None,
    };
    assert_eq!(response, expected_response);
}

#[test]
fn action_with_profit_deposit_to_anc() {
    let sdk = Sdk::init();
    let config = load_config(sdk.deps.as_ref().storage).unwrap();
    let tax_info = TaxInfo {
        rate: Decimal256::zero(),
        cap: Uint256::zero(),
    };

    let deposit_amount = Uint256::from(2_000u64);
    let action_with_profit = ActionWithProfit::DepositToAnc {
        amount: deposit_amount,
    };
    let response = action_with_profit.to_response(&config, &tax_info).unwrap();

    let stable_coin_to_lending: Uint128 = tax_info.subtract_tax(deposit_amount).into();
    let expected_response = Response {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: ANCHOR_MARKET_CONTRACT.to_string(),
            msg: to_binary(&AnchorMarketMsg::DepositStable {}).unwrap(),
            send: vec![Coin {
                denom: STABLE_DENOM.to_string(),
                amount: stable_coin_to_lending,
            }],
        })],
        submessages: vec![],
        attributes: vec![
            attr("action", "distribute_rewards"),
            attr("deposit_to_anc", stable_coin_to_lending),
        ],
        data: None,
    };
    assert_eq!(response, expected_response);
}

#[test]
fn action_with_profit_split() {
    let sdk = Sdk::init();
    let config = load_config(sdk.deps.as_ref().storage).unwrap();
    let tax_info = TaxInfo {
        rate: Decimal256::zero(),
        cap: Uint256::zero(),
    };

    let lending_amount = Uint256::from(2_000u64);
    let buy_psi_amount = Uint256::from(5_000u64);
    let action_with_profit = ActionWithProfit::Split {
        buy_psi: buy_psi_amount,
        deposit_to_anc: lending_amount,
    };
    let response = action_with_profit.to_response(&config, &tax_info).unwrap();

    let stable_coin_to_lending: Uint128 = tax_info.subtract_tax(lending_amount).into();
    let stable_coin_to_buy_psi: Uint128 = tax_info.subtract_tax(buy_psi_amount).into();
    let swap_asset = Asset {
        info: AssetInfo::NativeToken {
            denom: STABLE_DENOM.to_string(),
        },
        amount: stable_coin_to_buy_psi,
    };
    let expected_response = Response {
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: ANCHOR_MARKET_CONTRACT.to_string(),
                msg: to_binary(&AnchorMarketMsg::DepositStable {}).unwrap(),
                send: vec![Coin {
                    denom: STABLE_DENOM.to_string(),
                    amount: stable_coin_to_lending,
                }],
            }),
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
                    amount: stable_coin_to_buy_psi,
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
        ],
        submessages: vec![],
        attributes: vec![
            attr("action", "distribute_rewards"),
            attr("bying_psi", stable_coin_to_buy_psi),
            attr("deposit_to_anc", stable_coin_to_lending),
        ],
        data: None,
    };
    assert_eq!(response, expected_response);
}
