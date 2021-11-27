use super::sdk::Sdk;
use crate::error::ContractError;
use crate::state::{load_holder, load_state};
use cosmwasm_std::{Addr, StdError};
use cosmwasm_std::{Decimal, Uint128};
use std::str::FromStr;

#[test]
fn update_index_error_on_zero_nasset_amount() {
    let mut sdk = Sdk::init();

    //rewards balance is zero
    sdk.set_psi_balance(Uint128::zero());

    //===============================================================================
    let response = sdk.update_index();
    assert!(response.is_err());
    let error = response.err().unwrap();
    if let ContractError::Std(StdError::GenericErr { msg, .. }) = error {
        assert_eq!("nAsset balance is zero", msg);
    } else {
        panic!("wrong error");
    }
    //===============================================================================
}

#[test]
fn update_index_error_on_zero_rewards() {
    let mut sdk = Sdk::init();

    //rewards balance is zero
    sdk.set_psi_balance(Uint128::zero());

    //deposit some nAsset
    sdk.increase_user_balance(&Addr::unchecked("addr1000"), Uint128::new(200));

    //===============================================================================
    let response = sdk.update_index();
    assert!(response.is_err());
    let error = response.err().unwrap();
    if let ContractError::Std(StdError::GenericErr { msg, .. }) = error {
        assert_eq!("No rewards have accrued yet", msg);
    } else {
        panic!("wrong error");
    }
    //===============================================================================
}

#[test]
fn update_index_successfully_update() {
    let mut sdk = Sdk::init();

    //deposit some nAsset
    sdk.increase_user_balance(&Addr::unchecked("addr1000"), Uint128::new(200));

    //rewards balance is not zero
    sdk.set_psi_balance(Uint128::new(200));

    //===============================================================================
    let response = sdk.update_index();
    assert!(response.is_ok());
    //===============================================================================
}

#[test]
fn update_index_error_cause_increase_balance_already_update_it() {
    let mut sdk = Sdk::init();

    //rewards balance is not zero
    sdk.set_psi_balance(Uint128::new(200));

    //deposit some nAsset
    sdk.increase_user_balance(&Addr::unchecked("addr1000"), Uint128::new(200));

    //===============================================================================
    let response = sdk.update_index();
    assert!(response.is_err());
    //===============================================================================
}

#[test]
fn increase_balance_should_update_index() {
    let mut sdk = Sdk::init();
    let user_address = Addr::unchecked("addr1000".to_string());

    //rewards already there
    let rewards: Uint128 = 1000u64.into();
    sdk.set_psi_balance(rewards);

    let state = load_state(&sdk.deps.storage).unwrap();
    assert_eq!(Decimal::zero(), state.global_index);
    assert_eq!(Uint128::zero(), state.total_balance);
    assert_eq!(Uint128::zero(), state.prev_reward_balance);

    //===============================================================================
    //user deposit nasset
    //this action should update index

    let deposit_amount: Uint128 = 100u128.into();
    sdk.increase_user_balance(&user_address, deposit_amount);
    let holder_state = load_holder(&sdk.deps.storage, &user_address).unwrap();
    assert_eq!(deposit_amount, holder_state.balance);
    assert_eq!(Decimal::zero(), holder_state.index);
    assert_eq!(Decimal::zero(), holder_state.pending_rewards);

    let state = load_state(&sdk.deps.storage).unwrap();
    assert_eq!(Decimal::from_str("10").unwrap(), state.global_index);
    assert_eq!(deposit_amount, state.total_balance);
    assert_eq!(rewards, state.prev_reward_balance);
    //===============================================================================
}

#[test]
fn decrease_balance_should_update_index() {
    let mut sdk = Sdk::init();
    let user_address = Addr::unchecked("addr1000".to_string());

    //rewards balance is zero
    sdk.set_psi_balance(Uint128::zero());

    //===============================================================================
    //user deposit nasset

    let deposit_amount: Uint128 = 100u128.into();
    sdk.increase_user_balance(&user_address, deposit_amount);
    let holder_state = load_holder(&sdk.deps.storage, &user_address).unwrap();
    assert_eq!(deposit_amount, holder_state.balance);
    assert_eq!(Decimal::zero(), holder_state.index);
    assert_eq!(Decimal::zero(), holder_state.pending_rewards);

    sdk.query_holder_state(
        &user_address,
        deposit_amount,
        Decimal::zero(),
        Decimal::zero(),
    );

    //===============================================================================

    //rewards coming
    let rewards: Uint128 = 1000u64.into();
    sdk.set_psi_balance(rewards);

    sdk.query_holder_state(
        &user_address,
        deposit_amount,
        Decimal::from_ratio(rewards, deposit_amount),
        Decimal::from_ratio(rewards, Uint128::new(1)),
    );

    //===============================================================================
    //user send (decrease his amount) nasset
    //this action should update index

    let withdraw_amount: Uint128 = 50u128.into();
    sdk.decrease_user_balance(&user_address, withdraw_amount);
    let holder_state = load_holder(&sdk.deps.storage, &user_address).unwrap();
    assert_eq!(Uint128::new(50), holder_state.balance);
    assert_eq!(Decimal::from_str("10").unwrap(), holder_state.index);
    assert_eq!(
        Decimal::from_str("1000").unwrap(),
        holder_state.pending_rewards
    );

    let state = load_state(&sdk.deps.storage).unwrap();
    assert_eq!(Decimal::from_str("10").unwrap(), state.global_index);
    assert_eq!(Uint128::new(50), state.total_balance);
    assert_eq!(rewards, state.prev_reward_balance);

    sdk.query_holder_state(
        &user_address,
        deposit_amount.checked_sub(withdraw_amount).unwrap(),
        Decimal::from_ratio(rewards, deposit_amount),
        Decimal::from_ratio(rewards, Uint128::new(1)),
    )
    //===============================================================================
}
