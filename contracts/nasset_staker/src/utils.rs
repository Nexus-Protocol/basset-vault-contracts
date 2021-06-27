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
) -> Result<(), ContractError> {
    if state.total_staked_amount.is_zero() {
        return Ok(());
    }

    let psi_balance: Uint256 =
        query_token_balance(deps, &config.psi_token, &env.contract.address)?.into();

    calculate_reward_index(state, psi_balance)?;

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

fn calculate_reward_index(state: &mut State, psi_balance: Uint256) -> Result<(), ContractError> {
    if psi_balance < state.last_reward_amount {
        return Err(StdError::generic_err(
            "last nasset balance is bigger than current balance, impossible case",
        )
        .into());
    }

    let new_reward_amount: Uint256 = psi_balance - state.last_reward_amount;
    if new_reward_amount.is_zero() {
        return Ok(());
    }

    state.global_reward_index +=
        Decimal256::from_ratio(new_reward_amount.0, state.total_staked_amount.0);
    state.last_reward_amount = new_reward_amount;

    Ok(())
}

pub fn issue_reward(state: &mut State, staker_state: &mut StakerState) -> Uint256 {
    let claim_amount = staker_state.pending_rewards * Uint256::one();
    staker_state.pending_rewards =
        staker_state.pending_rewards - Decimal256::from_uint256(claim_amount);
    state.last_reward_amount = state.last_reward_amount - claim_amount;

    claim_amount
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
        let psi_balance = Uint256::zero();

        calculate_reward_index(&mut state, psi_balance).unwrap();
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
        let psi_balance = Uint256::from(200u64);

        calculate_reward_index(&mut state, psi_balance).unwrap();
        assert_eq!(state.last_reward_amount, Uint256::from(200u64));
        // 200 / 100
        assert_eq!(
            state.global_reward_index,
            Decimal256::from_str("2").unwrap()
        );
        assert_eq!(state.total_staked_amount, Uint256::from(100u64));
    }

    #[test]
    fn reward_calc_current_reward_lesser_than_previous() {
        let mut state = State {
            global_reward_index: Decimal256::from_str("245").unwrap(),
            last_reward_amount: Uint256::from(24_500u64),
            total_staked_amount: Uint256::from(1000u64),
        };
        let psi_balance = Uint256::from(8_000u64);

        let calc_res = calculate_reward_index(&mut state, psi_balance);
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
        let psi_balance = Uint256::from(80_000u64);

        calculate_reward_index(&mut state, psi_balance).unwrap();
        // 80k - 24_500
        assert_eq!(state.last_reward_amount, Uint256::from(55_500u64));
        //245 + 55_500 / 50_000
        assert_eq!(
            state.global_reward_index,
            Decimal256::from_str("246.11").unwrap(),
        );
        assert_eq!(state.total_staked_amount, Uint256::from(50_000u64));
    }
}
