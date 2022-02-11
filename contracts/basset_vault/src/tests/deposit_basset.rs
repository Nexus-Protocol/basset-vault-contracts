use super::sdk::Sdk;
use crate::tests::sdk::NASSET_TOKEN_ADDR;
use cosmwasm_bignumber::{Uint256, Decimal256};
use cosmwasm_std::{testing::MOCK_CONTRACT_ADDR, CosmosMsg};
use cosmwasm_std::{to_binary, StdError, SubMsg, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;

use basset_vault::basset_vault::{AnyoneMsg, ExecuteMsg};

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
                SubMsg::new(WasmMsg::Execute {
                    contract_addr: NASSET_TOKEN_ADDR.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Mint {
                        recipient: user_1_address.clone(),
                        amount: deposit_1_amount.into(),
                    })
                    .unwrap(),
                    funds: vec![],
                }),
                SubMsg::new(WasmMsg::Execute {
                    contract_addr: MOCK_CONTRACT_ADDR.to_string(),
                    msg: to_binary(&ExecuteMsg::Anyone {
                        anyone_msg: AnyoneMsg::Rebalance {},
                    })
                    .unwrap(),
                    funds: vec![],
                }),
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
    if let StdError::GenericErr { msg, .. } = response.err().unwrap() {
        assert_eq!(
            "bAsset balance is zero, but nAsset supply is not! Freeze contract.",
            msg
        );
    } else {
        panic!("wrong error");
    }
}

#[test]
fn deposit_basset_as_first_depositor_after_someone_transfer_some_bassets_directly_to_contract() {
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
                    contract_addr: NASSET_TOKEN_ADDR.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Mint {
                        recipient: user_address.clone(),
                        amount: total_basset_amount.into(), // first depositer have same amount
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
fn deposit_basset_after_someone_transfer_some_bassets_directly_to_contract_while_anchor() {
    let mut sdk = Sdk::init();

    //first farmer come
    let user_address = "addr9999".to_string();
    let deposited_earlier: Uint256 = 5_000_000_000u128.into();
    let deposit_amount: Uint256 = 2_000_000_000u128.into();
    let basset_directly_tranfered_amount: Uint256 = 10_000_000_000u128.into();
    let total_basset_amount = deposit_amount + basset_directly_tranfered_amount;
    {
        // -= USER SEND bAsset tokens to basset_vault =-
        sdk.set_nasset_supply(Uint256::zero());
        sdk.set_basset_balance(total_basset_amount);
        
        // Someone had deposited bAssets earlier and now they are in anchor
        sdk.set_collateral_balance(deposited_earlier);
        sdk.set_nasset_supply(deposited_earlier);

        let response = sdk
            .user_deposit(&user_address, deposit_amount.into())
            .unwrap();

        assert_eq!(
            response.messages,
            vec![
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: NASSET_TOKEN_ADDR.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Mint {
                        recipient: user_address.clone(),
                        amount: total_basset_amount.into(), // what user deposited + that someone deposited directly
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
fn deposit_basset_after_someone_transfer_some_bassets_directly_to_contract_while_holding() {
    let mut sdk = Sdk::init();

    //first farmer come
    let user_address = "addr9999".to_string();
    let deposited_earlier: Uint256 = 5_000_000_000u128.into();
    let deposit_amount: Uint256 = 2_000_000_000u128.into();
    let basset_directly_tranfered_amount: Uint256 = 10_000_000_000u128.into();
    let total_basset_amount = deposited_earlier + deposit_amount + basset_directly_tranfered_amount;
    {
        // -= USER SEND bAsset tokens to basset_vault =-
        sdk.set_nasset_supply(Uint256::zero());
        sdk.set_basset_balance(total_basset_amount);
        
        // Someone had deposited bAssets earlier and now vault is holding them
        sdk.set_nasset_supply(deposited_earlier);

        let response = sdk
            .user_deposit(&user_address, deposit_amount.into())
            .unwrap();

        let expected_nasset_to_mint = deposited_earlier * deposit_amount / Decimal256::from_uint256(deposited_earlier + basset_directly_tranfered_amount);

        assert_eq!(
            response.messages,
            vec![
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: NASSET_TOKEN_ADDR.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Mint {
                        recipient: user_address.clone(),
                        amount: expected_nasset_to_mint.into(),
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
