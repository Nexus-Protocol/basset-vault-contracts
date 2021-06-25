use crate::state::Config;
use crate::state::StakerState;
use crate::{error::ContractError, state::State};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Deps, Env, StdError};
use yield_optimizer::querier::query_token_balance;

pub fn update_global_reward(
    deps: Deps,
    env: Env,
    config: &Config,
    state: &mut State,
    incoming_staking_amount: Option<Uint256>,
) -> Result<(), ContractError> {
    if state.total_staked_amount.is_zero() {
        return Ok(());
    }

    let nasset_balance: Uint256 =
        query_token_balance(deps, &config.nasset_token, &env.contract.address)?.into();
    // balance already increased, so subtract deposit amount
    let balance_before_incoming_amount =
        nasset_balance - incoming_staking_amount.unwrap_or(Uint256::zero());

    calculate_reward_index(state, balance_before_incoming_amount)?;

    Ok(())
}

pub fn update_staker_reward(state: &State, staker_state: &mut StakerState) {
    let currently_staked = Decimal256::from_uint256(staker_state.staked_amount);
    staker_state.pending_rewards +=
        (state.global_reward_index - staker_state.reward_index) * currently_staked;
    staker_state.reward_index = state.global_reward_index;
}

pub fn increase_staked_amount(state: &mut State, staker_state: &mut StakerState, amount: Uint256) {
    state.total_staked_amount += amount;
    staker_state.staked_amount += amount;
}

pub fn decrease_staked_amount(state: &mut State, staker_state: &mut StakerState, amount: Uint256) {
    state.total_staked_amount = state.total_staked_amount - amount;
    staker_state.staked_amount = staker_state.staked_amount - amount;
}

fn calculate_reward_index(state: &mut State, nasset_balance: Uint256) -> Result<(), ContractError> {
    let last_balance = state.total_staked_amount + state.last_reward_amount;
    if nasset_balance < last_balance {
        return Err(StdError::generic_err(
            "last nasset balance is bigger than current balance, impossible case",
        )
        .into());
    }

    let new_reward_amount: Uint256 = nasset_balance - last_balance;
    if new_reward_amount.is_zero() {
        return Ok(());
    }

    state.global_reward_index +=
        Decimal256::from_ratio(new_reward_amount.0, state.total_staked_amount.0);
    state.last_reward_amount = new_reward_amount;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::calculate_reward_index;
    use crate::state::State;
    use cosmwasm_bignumber::{Decimal256, Uint256};
    use std::str::FromStr;

    #[test]
    fn reward_calc_zero_state() {
        let mut state = State {
            global_reward_index: Decimal256::zero(),
            last_reward_amount: Decimal256::zero(),
            total_staked_amount: Uint256::zero(),
        };
        let borrowed_amount = Uint256::from(0u64);
        let aterra_amount = Uint256::from(0u64);
        let aterra_exchange_rate = Decimal256::zero();
        let ust_amount = Uint256::from(0u64);

        calculate_reward_index(
            &mut state,
            borrowed_amount,
            aterra_amount,
            aterra_exchange_rate,
            ust_amount,
        );
        assert_eq!(state.last_reward_amount, Decimal256::zero());
        assert_eq!(state.global_reward_index, Decimal256::zero());
    }

    #[test]
    fn reward_calc_first_reward() {
        let mut state = State {
            global_reward_index: Decimal256::zero(),
            last_reward_amount: Decimal256::zero(),
            total_staked_amount: Uint256::from(100u64),
        };
        let borrowed_amount = Uint256::from(200_000u64);
        let aterra_amount = Uint256::from(195_000u64);
        let aterra_exchange_rate = Decimal256::from_str("1.1").unwrap();
        let ust_amount = Uint256::from(10_000u64);
        //195 * 1.1 = 214.5; 214.5 + 10 - 200 = 24.5

        calculate_reward_index(
            &mut state,
            borrowed_amount,
            aterra_amount,
            aterra_exchange_rate,
            ust_amount,
        );
        assert_eq!(
            state.last_reward_amount,
            Decimal256::from_str("24500").unwrap()
        );
        // 24_500 / 100
        assert_eq!(
            state.global_reward_index,
            Decimal256::from_str("245").unwrap()
        );
    }

    #[test]
    fn reward_calc_second_reward() {
        //get from 'first reward' test
        let mut state = State {
            global_reward_index: Decimal256::from_ratio(
                Uint256::from(24_500u64).0,
                Uint256::from(100u64).0,
            ),
            last_reward_amount: Decimal256::from_str("24500").unwrap(),
            total_staked_amount: Uint256::from(100u64),
        };
        let borrowed_amount = Uint256::from(300_000u64);
        let aterra_amount = Uint256::from(290_000u64);
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let ust_amount = Uint256::from(10_000u64);
        //290 * 1.2 = 348; 348 + 10 - 300 = 58

        let prev_global_reward_index = state.global_reward_index;
        calculate_reward_index(
            &mut state,
            borrowed_amount,
            aterra_amount,
            aterra_exchange_rate,
            ust_amount,
        );
        assert_eq!(
            state.last_reward_amount,
            Decimal256::from_str("58000").unwrap()
        );
        // new_rewards = 58_000 - 24_500 = 33_500
        assert_eq!(
            state.global_reward_index,
            prev_global_reward_index
                + Decimal256::from_ratio(Uint256::from(33_500u64).0, Uint256::from(100u64).0)
        );
    }

    #[test]
    fn reward_calc_borrowed_amount_bigger_then_aterra_amount() {
        let last_reward_amount = Decimal256::from_str("14500").unwrap();
        let global_reward_index =
            Decimal256::from_ratio(Uint256::from(24_500u64).0, Uint256::from(100u64).0);
        //get from 'first reward' test
        let mut state = State {
            global_reward_index,
            last_reward_amount,
            total_staked_amount: Uint256::from(100u64),
        };
        let borrowed_amount = Uint256::from(300_000u64);
        let aterra_amount = Uint256::from(200_000u64);
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let ust_amount = Uint256::from(10_000u64);
        //200 * 1.2 = 240; 240 + 10 - 300 = -50
        //so borrowed_amount is less than UST balance, means do not change rewards

        calculate_reward_index(
            &mut state,
            borrowed_amount,
            aterra_amount,
            aterra_exchange_rate,
            ust_amount,
        );
        assert_eq!(state.last_reward_amount, last_reward_amount);
        assert_eq!(state.global_reward_index, global_reward_index);
    }

    #[test]
    fn reward_calc_negative_reward() {
        let last_reward_amount = Decimal256::from_str("14500").unwrap();
        let global_reward_index =
            Decimal256::from_ratio(Uint256::from(24_500u64).0, Uint256::from(100u64).0);
        //get from 'first reward' test
        let mut state = State {
            global_reward_index,
            last_reward_amount,
            total_staked_amount: Uint256::from(100u64),
        };
        let borrowed_amount = Uint256::from(300_000u64);
        let aterra_amount = Uint256::from(275_000u64);
        let aterra_exchange_rate = Decimal256::from_str("1.1").unwrap();
        let ust_amount = Uint256::from(10_000u64);
        //275 * 1.1 = 302.5; 302.5 + 10 - 300 = 12.5
        //12.5 is less then previous reward amount, means do not change rewards

        calculate_reward_index(
            &mut state,
            borrowed_amount,
            aterra_amount,
            aterra_exchange_rate,
            ust_amount,
        );
        assert_eq!(state.last_reward_amount, last_reward_amount);
        assert_eq!(state.global_reward_index, global_reward_index);
    }
}
