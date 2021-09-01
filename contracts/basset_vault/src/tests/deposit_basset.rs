use super::sdk::Sdk;
use crate::tests::sdk::{
    ANCHOR_CUSTODY_BASSET_CONTRACT, ANCHOR_OVERSEER_CONTRACT, BASSET_TOKEN_ADDR, NASSET_TOKEN_ADDR,
};
use basset_vault::querier::AnchorCustodyCw20Msg;
use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{testing::MOCK_CONTRACT_ADDR, CosmosMsg};
use cosmwasm_std::{to_binary, StdError, SubMsg, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;

use basset_vault::{
    basset_vault::{AnyoneMsg, ExecuteMsg},
    querier::AnchorOverseerMsg,
};

#[test]
fn deposit_basset() {
    let mut sdk = Sdk::init();

    //first farmer come
    let user_1_address = "addr9999".to_string();
    let deposit_1_amount: Uint256 = 2_000_000_000u128.into();
    {
        // -= USER SEND bAsset tokens to basset_vault =-
        sdk.set_nasset_supply(Uint256::zero());
        sdk.set_basset_balance(deposit_1_amount);

        let response = sdk
            .user_deposit(&user_1_address, deposit_1_amount.into())
            .unwrap();

        assert_eq!(
            response.messages,
            vec![
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: BASSET_TOKEN_ADDR.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Send {
                        contract: ANCHOR_CUSTODY_BASSET_CONTRACT.to_string(),
                        amount: deposit_1_amount.into(),
                        msg: to_binary(&AnchorCustodyCw20Msg::DepositCollateral {}).unwrap()
                    })
                    .unwrap(),
                    funds: vec![],
                })),
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: ANCHOR_OVERSEER_CONTRACT.to_string(),
                    msg: to_binary(&AnchorOverseerMsg::LockCollateral {
                        collaterals: vec![(BASSET_TOKEN_ADDR.to_string(), deposit_1_amount)],
                    })
                    .unwrap(),
                    funds: vec![],
                })),
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: NASSET_TOKEN_ADDR.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Mint {
                        recipient: user_1_address.clone(),
                        amount: deposit_1_amount.into(), //first depositer have same amount
                    })
                    .unwrap(),
                    funds: vec![],
                })),
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: MOCK_CONTRACT_ADDR.to_string(),
                    msg: to_binary(&ExecuteMsg::Anyone {
                        anyone_msg: AnyoneMsg::Rebalance {},
                    })
                    .unwrap(),
                    funds: vec![],
                })),
            ]
        );
    }

    //second farmer come
    let user_2_address = "addr6666".to_string();
    let deposit_2_amount: Uint256 = 6_000_000_000u128.into();
    {
        sdk.set_nasset_supply(deposit_1_amount);
        sdk.set_collateral_balance(deposit_1_amount);
        sdk.set_basset_balance(deposit_2_amount);
        // -= USER SEND bAsset tokens to basset_vault =-
        let response = sdk
            .user_deposit(&user_2_address, deposit_2_amount.into())
            .unwrap();
        assert_eq!(
            response.messages,
            vec![
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: BASSET_TOKEN_ADDR.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Send {
                        contract: ANCHOR_CUSTODY_BASSET_CONTRACT.to_string(),
                        amount: deposit_2_amount.into(),
                        msg: to_binary(&AnchorCustodyCw20Msg::DepositCollateral {}).unwrap()
                    })
                    .unwrap(),
                    funds: vec![],
                })),
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: ANCHOR_OVERSEER_CONTRACT.to_string(),
                    msg: to_binary(&AnchorOverseerMsg::LockCollateral {
                        collaterals: vec![(BASSET_TOKEN_ADDR.to_string(), deposit_2_amount)],
                    })
                    .unwrap(),
                    funds: vec![],
                })),
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: NASSET_TOKEN_ADDR.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Mint {
                        recipient: user_2_address.clone(),
                        amount: Uint128::new(6_000_000_000), //2B * (6B/8B) / (1 - (6B/8B)) = 6B
                    })
                    .unwrap(),
                    funds: vec![],
                })),
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: MOCK_CONTRACT_ADDR.to_string(),
                    msg: to_binary(&ExecuteMsg::Anyone {
                        anyone_msg: AnyoneMsg::Rebalance {},
                    })
                    .unwrap(),
                    funds: vec![],
                })),
            ]
        );
    }
}

#[test]
fn do_not_accept_deposit_if_nluna_supply_is_not_zero_but_bluna_in_custody_is_zero() {
    let mut sdk = Sdk::init();

    sdk.set_nasset_supply(Uint256::one());
    sdk.set_collateral_balance(Uint256::zero());

    //farmer comes
    let user_address = "addr9999".to_string();
    let deposit_amount: Uint256 = 2_000_000_000u128.into();
    // -= USER SEND bAsset tokens to basset_vault =-
    sdk.set_basset_balance(deposit_amount);

    let response = sdk.user_deposit(&user_address, deposit_amount.into());
    assert!(response.is_err());
    if let StdError::GenericErr { msg } = response.err().unwrap() {
        assert_eq!(
            "bAsset balance is zero, but nAsset supply is not! Freeze contract.",
            msg
        );
    } else {
        panic!("wrong error");
    }
}

#[test]
fn deposit_basset_after_someone_transfer_some_bassets_directly_to_contract() {
    let mut sdk = Sdk::init();

    //first farmer come
    let user_address = "addr9999".to_string();
    let deposit_amount: Uint256 = 2_000_000_000u128.into();
    let basset_directly_tranfered_amount: Uint256 = 10_000_000_000u128.into();
    let total_basset_amount = deposit_amount + basset_directly_tranfered_amount;
    {
        // -= USER SEND bAsset tokens to basset_vault =-
        sdk.set_nasset_supply(Uint256::zero());
        sdk.set_basset_balance(total_basset_amount);

        let response = sdk
            .user_deposit(&user_address, deposit_amount.into())
            .unwrap();

        assert_eq!(
            response.messages,
            vec![
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: BASSET_TOKEN_ADDR.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Send {
                        contract: ANCHOR_CUSTODY_BASSET_CONTRACT.to_string(),
                        amount: total_basset_amount.into(),
                        msg: to_binary(&AnchorCustodyCw20Msg::DepositCollateral {}).unwrap()
                    })
                    .unwrap(),
                    funds: vec![],
                })),
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: ANCHOR_OVERSEER_CONTRACT.to_string(),
                    msg: to_binary(&AnchorOverseerMsg::LockCollateral {
                        collaterals: vec![(BASSET_TOKEN_ADDR.to_string(), total_basset_amount)],
                    })
                    .unwrap(),
                    funds: vec![],
                })),
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: NASSET_TOKEN_ADDR.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Mint {
                        recipient: user_address.clone(),
                        amount: total_basset_amount.into(), //first depositer have same amount
                    })
                    .unwrap(),
                    funds: vec![],
                })),
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: MOCK_CONTRACT_ADDR.to_string(),
                    msg: to_binary(&ExecuteMsg::Anyone {
                        anyone_msg: AnyoneMsg::Rebalance {},
                    })
                    .unwrap(),
                    funds: vec![],
                })),
            ]
        );
    }
}
