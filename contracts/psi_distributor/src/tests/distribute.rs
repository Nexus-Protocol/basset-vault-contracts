use super::sdk::Sdk;
use crate::{
    error::ContractError,
    tests::sdk::{
        COMMUNITY_POOL_CONTRACT_ADDR, GOVERNANCE_CONTRACT_ADDR, NASSET_TOKEN_REWARDS_CONTRACT_ADDR,
        PSI_TOKEN_ADDR,
    },
};
use basset_vault::nasset_token_rewards::{
    AnyoneMsg as NAssetTokenRewardsAnyoneMsg, ExecuteMsg as NAssetTokenRewardsExecuteMsg,
};
use cosmwasm_std::{to_binary, StdError, Uint128};
use cosmwasm_std::{CosmosMsg, SubMsg, WasmMsg};
use cw20::Cw20ExecuteMsg;

#[test]
fn distribute_rewards() {
    let mut sdk = Sdk::init();

    //contract have some "rewards"
    sdk.set_psi_balance(Uint128::new(1000));

    //===============================================================================
    //distribute rewards

    let response = sdk.distribute_rewards().unwrap();
    assert_eq!(
        response.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: PSI_TOKEN_ADDR.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: NASSET_TOKEN_REWARDS_CONTRACT_ADDR.to_string(),
                    amount: Uint128::from(900u64),
                })
                .unwrap(),
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: PSI_TOKEN_ADDR.to_string(),
                funds: vec![],
                msg: to_binary(&NAssetTokenRewardsExecuteMsg::Anyone {
                    anyone_msg: NAssetTokenRewardsAnyoneMsg::UpdateGlobalIndex {},
                })
                .unwrap(),
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: PSI_TOKEN_ADDR.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: GOVERNANCE_CONTRACT_ADDR.to_string(),
                    amount: Uint128::from(75u64),
                })
                .unwrap(),
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: PSI_TOKEN_ADDR.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: COMMUNITY_POOL_CONTRACT_ADDR.to_string(),
                    amount: Uint128::from(25u64),
                })
                .unwrap(),
            })),
        ]
    );
    //===============================================================================
}

#[test]
fn distribute_rewards_enought_only_for_nasset_holders() {
    let mut sdk = Sdk::init();

    //contract have some "rewards"
    sdk.set_psi_balance(Uint128::new(9));

    //===============================================================================
    //distribute rewards

    let response = sdk.distribute_rewards().unwrap();
    assert_eq!(
        response.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: PSI_TOKEN_ADDR.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: NASSET_TOKEN_REWARDS_CONTRACT_ADDR.to_string(),
                    amount: Uint128::from(9u64),
                })
                .unwrap(),
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: PSI_TOKEN_ADDR.to_string(),
                funds: vec![],
                msg: to_binary(&NAssetTokenRewardsExecuteMsg::Anyone {
                    anyone_msg: NAssetTokenRewardsAnyoneMsg::UpdateGlobalIndex {},
                })
                .unwrap(),
            })),
        ]
    );
    //===============================================================================
}

#[test]
fn distribute_rewards_enought_for_nasset_holders_and_governance() {
    let mut sdk = Sdk::init();

    //contract have some "rewards"
    sdk.set_psi_balance(Uint128::new(10));

    //===============================================================================
    //distribute rewards

    let response = sdk.distribute_rewards().unwrap();
    assert_eq!(
        response.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: PSI_TOKEN_ADDR.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: NASSET_TOKEN_REWARDS_CONTRACT_ADDR.to_string(),
                    amount: Uint128::from(9u64),
                })
                .unwrap(),
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: PSI_TOKEN_ADDR.to_string(),
                funds: vec![],
                msg: to_binary(&NAssetTokenRewardsExecuteMsg::Anyone {
                    anyone_msg: NAssetTokenRewardsAnyoneMsg::UpdateGlobalIndex {},
                })
                .unwrap(),
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: PSI_TOKEN_ADDR.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: GOVERNANCE_CONTRACT_ADDR.to_string(),
                    amount: Uint128::from(1u64),
                })
                .unwrap(),
            })),
        ]
    );
    //===============================================================================
}

#[test]
fn distribute_rewards_with_zero_balance() {
    let mut sdk = Sdk::init();

    //===============================================================================
    //distribute rewards

    let response = sdk.distribute_rewards();
    assert!(response.is_err());
    let error = response.err().unwrap();
    if let ContractError::Std(StdError::GenericErr { msg }) = error {
        assert_eq!("psi balance is zero", msg);
    } else {
        panic!("wrong error");
    }
    //===============================================================================
}
