use super::sdk::Sdk;
use crate::tests::sdk::PSI_TOKEN_ADDR;
use crate::{
    error::ContractError,
    state::{load_holder, load_state},
};
use cosmwasm_std::{to_binary, SubMsg, Uint128};
use cosmwasm_std::{Addr, CosmosMsg, Decimal, StdError, WasmMsg};
use cw20::Cw20ExecuteMsg;
use std::str::FromStr;

#[test]
fn increase_balance_and_claim_rewards() {
    let mut sdk = Sdk::init();
    let user_1_address = Addr::unchecked("addr1000".to_string());

    //rewards already there
    let rewards_before_receive_nasset: Uint128 = 1000u64.into();
    sdk.set_psi_balance(rewards_before_receive_nasset);

    //===============================================================================
    //first user deposit nasset

    let deposit_1_amount: Uint128 = 100u128.into();
    {
        sdk.increase_user_balance(&user_1_address, deposit_1_amount);
        let holder_state = load_holder(&sdk.deps.storage, &user_1_address).unwrap();
        assert_eq!(deposit_1_amount, holder_state.balance);
        assert_eq!(Decimal::zero(), holder_state.index);
        assert_eq!(Decimal::zero(), holder_state.pending_rewards);

        sdk.query_holder_state(
            &user_1_address,
            deposit_1_amount,
            Decimal::from_ratio(rewards_before_receive_nasset, deposit_1_amount),
            Decimal::from_ratio(rewards_before_receive_nasset, Uint128::new(1)),
        )
    }

    //===============================================================================
    //rewards incoming

    let rewards_after_receive_nasset: Uint128 = 5000u64.into();
    {
        sdk.set_psi_balance(rewards_before_receive_nasset + rewards_after_receive_nasset);
        sdk.query_holder_state(
            &user_1_address,
            deposit_1_amount,
            Decimal::from_str("60").unwrap(),
            Decimal::from_ratio(
                rewards_before_receive_nasset + rewards_after_receive_nasset,
                Uint128::new(1),
            ),
        )
    }

    //===============================================================================
    //first user claim rewards

    {
        let response = sdk.claim_rewards(&user_1_address).unwrap();
        sdk.set_psi_balance(Uint128::zero());

        assert_eq!(
            response.messages,
            vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: PSI_TOKEN_ADDR.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: user_1_address.to_string(),
                    amount: rewards_before_receive_nasset + rewards_after_receive_nasset,
                })
                .unwrap(),
            }))]
        );

        let holder = load_holder(&sdk.deps.storage, &user_1_address).unwrap();
        assert_eq!(deposit_1_amount, holder.balance);
        assert_eq!(Decimal::from_str("60").unwrap(), holder.index);
        assert_eq!(Decimal::zero(), holder.pending_rewards);

        let state = load_state(&sdk.deps.storage).unwrap();
        assert_eq!(Decimal::from_str("60").unwrap(), state.global_index);
        assert_eq!(deposit_1_amount, state.total_balance);
        assert_eq!(Uint128::zero(), state.prev_reward_balance);

        sdk.query_holder_state(
            &user_1_address,
            deposit_1_amount,
            Decimal::from_str("60").unwrap(),
            Decimal::zero(),
        )
    }
    //===============================================================================
}

#[test]
fn claim_zero_rewards() {
    let mut sdk = Sdk::init();
    let user_1_address = Addr::unchecked("addr1000".to_string());

    //rewards already there
    let rewards: Uint128 = Uint128::zero();
    sdk.set_psi_balance(rewards);

    //===============================================================================
    //first user deposit nasset

    let deposit_1_amount: Uint128 = 100u128.into();
    {
        sdk.increase_user_balance(&user_1_address, deposit_1_amount);
        let holder_state = load_holder(&sdk.deps.storage, &user_1_address).unwrap();
        assert_eq!(deposit_1_amount, holder_state.balance);
        assert_eq!(Decimal::zero(), holder_state.index);
        assert_eq!(Decimal::zero(), holder_state.pending_rewards);
    }

    //===============================================================================
    //first user claim rewards

    {
        let response = sdk.claim_rewards(&user_1_address);

        assert!(response.is_err());
        let error = response.err().unwrap();
        if let ContractError::Std(StdError::GenericErr { msg }) = error {
            assert_eq!("No rewards have accrued yet", msg);
        } else {
            panic!("wrong error type");
        };

        let holder = load_holder(&sdk.deps.storage, &user_1_address).unwrap();
        assert_eq!(deposit_1_amount, holder.balance);
        assert_eq!(Decimal::zero(), holder.index);
        assert_eq!(Decimal::zero(), holder.pending_rewards);

        let state = load_state(&sdk.deps.storage).unwrap();
        assert_eq!(Decimal::zero(), state.global_index);
        assert_eq!(deposit_1_amount, state.total_balance);
        assert_eq!(Uint128::zero(), state.prev_reward_balance);
    }
    //===============================================================================
}

#[test]
fn second_user_comes_after_rewards_already_there() {
    let mut sdk = Sdk::init();
    let user_1_address = Addr::unchecked("addr1000".to_string());
    let user_2_address = Addr::unchecked("addr1001".to_string());

    //rewards already there
    let rewards_before_receive_nasset: Uint128 = 1000u64.into();
    sdk.set_psi_balance(rewards_before_receive_nasset);

    //===============================================================================
    //first user deposit nasset

    let deposit_1_amount: Uint128 = 100u128.into();
    {
        sdk.increase_user_balance(&user_1_address, deposit_1_amount);
        let holder_state = load_holder(&sdk.deps.storage, &user_1_address).unwrap();
        assert_eq!(deposit_1_amount, holder_state.balance);
        assert_eq!(Decimal::zero(), holder_state.index);
        assert_eq!(Decimal::zero(), holder_state.pending_rewards);
    }

    //===============================================================================
    //second user deposit nasset
    //existed rewards belong to first user

    let deposit_2_amount: Uint128 = 300u128.into();
    {
        sdk.increase_user_balance(&user_2_address, deposit_2_amount);
        let holder_state = load_holder(&sdk.deps.storage, &user_2_address).unwrap();
        assert_eq!(deposit_2_amount, holder_state.balance);
        assert_eq!(Decimal::from_str("10").unwrap(), holder_state.index);
        assert_eq!(Decimal::zero(), holder_state.pending_rewards);
    }

    //===============================================================================
    //rewards incoming

    let rewards_after_receive_nasset: Uint128 = 5000u64.into();
    {
        sdk.set_psi_balance(rewards_before_receive_nasset + rewards_after_receive_nasset);
    }

    //===============================================================================
    //first user claim rewards
    let rewards_1_amount = Uint128::new(2250);
    {
        let response = sdk.claim_rewards(&user_1_address).unwrap();

        assert_eq!(
            response.messages,
            vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: PSI_TOKEN_ADDR.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: user_1_address.to_string(),
                    amount: rewards_1_amount, // total_rewards = 1000 + 5k * share (1/4)
                })
                .unwrap(),
            }))]
        );

        //22.5 comes from: (first_user_reward / first_balance) + new_rewards / total_balance
        //so, 1000 / 100 + 5000 / 400
        let holder = load_holder(&sdk.deps.storage, &user_1_address).unwrap();
        assert_eq!(deposit_1_amount, holder.balance);
        assert_eq!(Decimal::from_str("22.5").unwrap(), holder.index);
        assert_eq!(Decimal::zero(), holder.pending_rewards);

        let state = load_state(&sdk.deps.storage).unwrap();
        assert_eq!(Decimal::from_str("22.5").unwrap(), state.global_index);
        assert_eq!(deposit_1_amount + deposit_2_amount, state.total_balance);
        assert_eq!(Uint128::new(3750), state.prev_reward_balance);
    }

    //===============================================================================
    // subtract rewards sent to user_1

    sdk.set_psi_balance(
        (rewards_before_receive_nasset + rewards_after_receive_nasset)
            .checked_sub(rewards_1_amount)
            .unwrap(),
    );

    //===============================================================================
    //second user claim rewards

    {
        let response = sdk.claim_rewards(&user_2_address).unwrap();

        assert_eq!(
            response.messages,
            vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: PSI_TOKEN_ADDR.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: user_2_address.to_string(),
                    amount: Uint128::new(3750), // total_rewards = 5k * share (3/4)
                })
                .unwrap(),
            }))]
        );

        let holder = load_holder(&sdk.deps.storage, &user_2_address).unwrap();
        assert_eq!(deposit_2_amount, holder.balance);
        assert_eq!(Decimal::from_str("22.5").unwrap(), holder.index);
        assert_eq!(Decimal::zero(), holder.pending_rewards);

        let state = load_state(&sdk.deps.storage).unwrap();
        assert_eq!(Decimal::from_str("22.5").unwrap(), state.global_index);
        assert_eq!(deposit_1_amount + deposit_2_amount, state.total_balance);
        assert_eq!(Uint128::zero(), state.prev_reward_balance);
    }
    //===============================================================================
}

#[test]
fn two_users_hold_partially_unhold_and_hold_again() {
    let mut sdk = Sdk::init();
    let user_1_address = Addr::unchecked("addr1000".to_string());
    let user_2_address = Addr::unchecked("addr1001".to_string());

    //===============================================================================
    //first user deposit nasset

    let deposit_1_amount: Uint128 = 100u128.into();
    {
        sdk.increase_user_balance(&user_1_address, deposit_1_amount);
        let holder_state = load_holder(&sdk.deps.storage, &user_1_address).unwrap();
        assert_eq!(deposit_1_amount, holder_state.balance);
        assert_eq!(Decimal::zero(), holder_state.index);
        assert_eq!(Decimal::zero(), holder_state.pending_rewards);
    }

    //===============================================================================
    //second user deposit nasset

    let deposit_2_amount: Uint128 = 300u128.into();
    {
        sdk.increase_user_balance(&user_2_address, deposit_2_amount);
        let holder_state = load_holder(&sdk.deps.storage, &user_2_address).unwrap();
        assert_eq!(deposit_2_amount, holder_state.balance);
        assert_eq!(Decimal::zero(), holder_state.index);
        assert_eq!(Decimal::zero(), holder_state.pending_rewards);
    }

    //===============================================================================

    let new_rewards_amount: Uint128 = 1000u64.into();
    sdk.set_psi_balance(new_rewards_amount);

    //===============================================================================
    //first user withdraw

    let withdraw_1_amount: Uint128 = 30u128.into();
    sdk.decrease_user_balance(&user_1_address, withdraw_1_amount);

    // 1/4(total stake share) * 1000 (psi balance)
    let rewards_1_amount = Uint128::from(250u64);
    let holder_state = load_holder(&sdk.deps.storage, &user_1_address).unwrap();
    assert_eq!(
        deposit_1_amount.checked_sub(withdraw_1_amount).unwrap(),
        holder_state.balance
    );
    assert_eq!(Decimal::from_str("2.5").unwrap(), holder_state.index);
    //it is 'rewards_1_amount'
    assert_eq!(
        Decimal::from_str("250").unwrap(),
        holder_state.pending_rewards
    );

    let state = load_state(&sdk.deps.storage).unwrap();
    assert_eq!(Decimal::from_str("2.5").unwrap(), state.global_index);
    assert_eq!(
        (deposit_1_amount + deposit_2_amount)
            .checked_sub(withdraw_1_amount)
            .unwrap(),
        state.total_balance
    );
    assert_eq!(new_rewards_amount, state.prev_reward_balance);

    //===============================================================================
    //first user claim rewards

    {
        let response = sdk.claim_rewards(&user_1_address).unwrap();

        assert_eq!(
            response.messages,
            vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: PSI_TOKEN_ADDR.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: user_1_address.to_string(),
                    amount: rewards_1_amount,
                })
                .unwrap(),
            }))]
        );

        let holder = load_holder(&sdk.deps.storage, &user_1_address).unwrap();
        assert_eq!(Decimal::zero(), holder.pending_rewards);

        let state = load_state(&sdk.deps.storage).unwrap();
        assert_eq!(
            new_rewards_amount.checked_sub(rewards_1_amount).unwrap(),
            state.prev_reward_balance
        );
    }

    //===============================================================================
    // subtract rewards sent to user_1

    sdk.set_psi_balance(new_rewards_amount.checked_sub(rewards_1_amount).unwrap());

    //===============================================================================
    //second user withdraw

    let withdraw_2_amount: Uint128 = 150u128.into();
    sdk.decrease_user_balance(&user_2_address, withdraw_2_amount);

    // 1/4(total stake share) * 1000 (dep1+ dep2)
    let rewards_2_amount = Uint128::from(750u64);
    let holder_state = load_holder(&sdk.deps.storage, &user_2_address).unwrap();
    assert_eq!(
        deposit_2_amount.checked_sub(withdraw_2_amount).unwrap(),
        holder_state.balance
    );
    assert_eq!(Decimal::from_str("2.5").unwrap(), holder_state.index);
    //it is 'rewards_2_amount'
    assert_eq!(
        Decimal::from_str("750").unwrap(),
        holder_state.pending_rewards
    );

    let state = load_state(&sdk.deps.storage).unwrap();
    assert_eq!(Decimal::from_str("2.5").unwrap(), state.global_index);
    assert_eq!(
        (deposit_1_amount + deposit_2_amount)
            .checked_sub(withdraw_1_amount)
            .unwrap()
            .checked_sub(withdraw_2_amount)
            .unwrap(),
        state.total_balance
    );
    assert_eq!(
        new_rewards_amount.checked_sub(rewards_1_amount).unwrap(),
        state.prev_reward_balance
    );

    //===============================================================================
    //second user claim rewards

    {
        let response = sdk.claim_rewards(&user_2_address).unwrap();

        assert_eq!(
            response.messages,
            vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: PSI_TOKEN_ADDR.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: user_2_address.to_string(),
                    amount: rewards_2_amount,
                })
                .unwrap(),
            }))]
        );

        let holder = load_holder(&sdk.deps.storage, &user_2_address).unwrap();
        assert_eq!(Decimal::zero(), holder.pending_rewards);

        let state = load_state(&sdk.deps.storage).unwrap();
        assert_eq!(Uint128::zero(), state.prev_reward_balance);
    }

    //===============================================================================
    // subtract rewards sent to user_1 and user_2

    sdk.set_psi_balance(Uint128::zero());

    //===============================================================================
    //first user deposit second time

    let second_deposit_1_amount = Uint128::from(130u64);
    {
        sdk.increase_user_balance(&user_1_address, second_deposit_1_amount);
        let holder_state = load_holder(&sdk.deps.storage, &user_1_address).unwrap();
        assert_eq!(
            (deposit_1_amount + second_deposit_1_amount)
                .checked_sub(withdraw_1_amount)
                .unwrap(),
            holder_state.balance
        );
        assert_eq!(Decimal::from_str("2.5").unwrap(), holder_state.index);
        assert_eq!(Decimal::zero(), holder_state.pending_rewards);

        let state = load_state(&sdk.deps.storage).unwrap();
        assert_eq!(Decimal::from_str("2.5").unwrap(), state.global_index);
        assert_eq!(
            (deposit_1_amount + deposit_2_amount + second_deposit_1_amount)
                .checked_sub(withdraw_1_amount)
                .unwrap()
                .checked_sub(withdraw_2_amount)
                .unwrap(),
            state.total_balance
        );
        assert_eq!(Uint128::zero(), state.prev_reward_balance);
    }

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
    //===============================================================================

    //rewards coming
    let rewards: Uint128 = 1000u64.into();
    sdk.set_psi_balance(rewards);

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
    //===============================================================================
}
