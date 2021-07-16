use crate::{state::query_external_config_light, tax_querier::TaxInfo, utils::ActionWithProfit};

use super::sdk::Sdk;
use crate::{
    state::load_config,
    tests::sdk::{
        ANCHOR_MARKET_CONTRACT, PSI_DISTRIBUTOR_CONTRACT, PSI_STABLE_SWAP_CONTRACT, STABLE_DENOM,
    },
};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{attr, CosmosMsg, SubMsg};
use cosmwasm_std::{to_binary, Coin, Response, Uint128, WasmMsg};

use basset_vault::{
    psi_distributor::{
        AnyoneMsg as PsiDistributorAnyoneMsg, ExecuteMsg as PsiDistributorExecuteMsg,
    },
    querier::AnchorMarketMsg,
    terraswap::{Asset, AssetInfo},
    terraswap_pair::ExecuteMsg as TerraswapExecuteMsg,
};

#[test]
fn action_with_profit_nothing() {
    let sdk = Sdk::init();
    let config = load_config(sdk.deps.as_ref().storage).unwrap();
    let tax_info = TaxInfo {
        rate: Decimal256::zero(),
        cap: Uint256::zero(),
    };
    let external_config = query_external_config_light(sdk.deps.as_ref(), &config).unwrap();

    let action_with_profit = ActionWithProfit::Nothing;
    let response = action_with_profit
        .to_response(&config, &external_config, &tax_info)
        .unwrap();
    assert_eq!(
        response,
        Response {
            messages: vec![],
            events: vec![],
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
    let external_config = query_external_config_light(sdk.deps.as_ref(), &config).unwrap();
    let tax_info = TaxInfo {
        rate: Decimal256::zero(),
        cap: Uint256::zero(),
    };

    let buy_psi_amount = Uint256::from(2_000u64);
    let action_with_profit = ActionWithProfit::BuyPsi {
        amount: buy_psi_amount,
    };
    let response = action_with_profit
        .to_response(&config, &external_config, &tax_info)
        .unwrap();

    let swap_asset = Asset {
        info: AssetInfo::NativeToken {
            denom: STABLE_DENOM.to_string(),
        },
        amount: tax_info.subtract_tax(buy_psi_amount).into(),
    };
    let expected_response = Response {
        messages: vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: PSI_STABLE_SWAP_CONTRACT.to_string(),
                msg: to_binary(&TerraswapExecuteMsg::Swap {
                    offer_asset: swap_asset,
                    max_spread: None,
                    belief_price: None,
                    to: Some(PSI_DISTRIBUTOR_CONTRACT.to_string()),
                })
                .unwrap(),
                funds: vec![Coin {
                    denom: STABLE_DENOM.to_string(),
                    amount: buy_psi_amount.into(),
                }],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: PSI_DISTRIBUTOR_CONTRACT.to_string(),
                msg: to_binary(&PsiDistributorExecuteMsg::Anyone {
                    anyone_msg: PsiDistributorAnyoneMsg::DistributeRewards,
                })
                .unwrap(),
                funds: vec![],
            })),
        ],
        events: vec![],
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
    let external_config = query_external_config_light(sdk.deps.as_ref(), &config).unwrap();
    let tax_info = TaxInfo {
        rate: Decimal256::zero(),
        cap: Uint256::zero(),
    };

    let deposit_amount = Uint256::from(2_000u64);
    let action_with_profit = ActionWithProfit::DepositToAnc {
        amount: deposit_amount,
    };
    let response = action_with_profit
        .to_response(&config, &external_config, &tax_info)
        .unwrap();

    let stable_coin_to_lending: Uint128 = tax_info.subtract_tax(deposit_amount).into();
    let expected_response = Response {
        messages: vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: ANCHOR_MARKET_CONTRACT.to_string(),
            msg: to_binary(&AnchorMarketMsg::DepositStable).unwrap(),
            funds: vec![Coin {
                denom: STABLE_DENOM.to_string(),
                amount: stable_coin_to_lending,
            }],
        }))],
        events: vec![],
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
    let external_config = query_external_config_light(sdk.deps.as_ref(), &config).unwrap();
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
    let response = action_with_profit
        .to_response(&config, &external_config, &tax_info)
        .unwrap();

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
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: ANCHOR_MARKET_CONTRACT.to_string(),
                msg: to_binary(&AnchorMarketMsg::DepositStable).unwrap(),
                funds: vec![Coin {
                    denom: STABLE_DENOM.to_string(),
                    amount: stable_coin_to_lending,
                }],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: PSI_STABLE_SWAP_CONTRACT.to_string(),
                msg: to_binary(&TerraswapExecuteMsg::Swap {
                    offer_asset: swap_asset,
                    max_spread: None,
                    belief_price: None,
                    to: Some(PSI_DISTRIBUTOR_CONTRACT.to_string()),
                })
                .unwrap(),
                funds: vec![Coin {
                    denom: STABLE_DENOM.to_string(),
                    amount: stable_coin_to_buy_psi,
                }],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: PSI_DISTRIBUTOR_CONTRACT.to_string(),
                msg: to_binary(&PsiDistributorExecuteMsg::Anyone {
                    anyone_msg: PsiDistributorAnyoneMsg::DistributeRewards,
                })
                .unwrap(),
                funds: vec![],
            })),
        ],
        events: vec![],
        attributes: vec![
            attr("action", "distribute_rewards"),
            attr("bying_psi", stable_coin_to_buy_psi),
            attr("deposit_to_anc", stable_coin_to_lending),
        ],
        data: None,
    };
    assert_eq!(response, expected_response);
}
