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
            last_reward_amount: Uint256::zero(),
            total_staked_amount: Uint256::zero(),
        };
        let nasset_balance = Uint256::zero();

        calculate_reward_index(&mut state, nasset_balance).unwrap();
        assert_eq!(state.last_reward_amount, Uint256::zero());
        assert_eq!(state.global_reward_index, Decimal256::zero());
        assert_eq!(state.total_staked_amount, Uint256::zero());
    }

    #[test]
    fn reward_calc_first_reward() {
        let mut state = State {
            global_reward_index: Decimal256::zero(),
            last_reward_amount: Uint256::zero(),
            total_staked_amount: Uint256::from(100u64),
        };
        let nasset_balance = Uint256::from(200u64);

        calculate_reward_index(&mut state, nasset_balance).unwrap();
        assert_eq!(state.last_reward_amount, Uint256::from(100u64));
        // (200 - 100) / 100
        assert_eq!(state.global_reward_index, Decimal256::one());
        assert_eq!(state.total_staked_amount, Uint256::from(100u64));
    }

    #[test]
    fn reward_calc_current_balance_lesser_than_staked() {
        let mut state = State {
            global_reward_index: Decimal256::zero(),
            last_reward_amount: Uint256::zero(),
            total_staked_amount: Uint256::from(100u64),
        };
        let nasset_balance = Uint256::from(70u64);

        let calc_res = calculate_reward_index(&mut state, nasset_balance);
        assert!(calc_res.is_err());
        assert_eq!(state.total_staked_amount, Uint256::from(100u64));
    }

    #[test]
    fn reward_calc_current_reward_lesser_than_previous() {
        let mut state = State {
            global_reward_index: Decimal256::from_str("245").unwrap(),
            last_reward_amount: Uint256::from(24_500u64),
            total_staked_amount: Uint256::from(1000u64),
        };
        let nasset_balance = Uint256::from(8_000u64);

        let calc_res = calculate_reward_index(&mut state, nasset_balance);
        assert!(calc_res.is_err());
        assert_eq!(state.total_staked_amount, Uint256::from(1000u64));
    }

    #[test]
    fn reward_calc_second_reward() {
        let mut state = State {
            global_reward_index: Decimal256::from_str("245").unwrap(),
            last_reward_amount: Uint256::from(24_500u64),
            total_staked_amount: Uint256::from(50_000u64),
        };
        let nasset_balance = Uint256::from(80_000u64);

        calculate_reward_index(&mut state, nasset_balance).unwrap();
        assert_eq!(Uint256::from(5_500u64), state.last_reward_amount);
        //245 + 5_500 / 50_000
        assert_eq!(
            Decimal256::from_str("245.11").unwrap(),
            state.global_reward_index
        );
        assert_eq!(state.total_staked_amount, Uint256::from(50_000u64));
    }
}
