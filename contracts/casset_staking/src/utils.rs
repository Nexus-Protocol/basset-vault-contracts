use crate::state::Config;
use crate::state::StakerState;
use crate::{error::ContractError, state::State};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Deps, Env};
use yield_optimizer::querier::{
    query_aterra_state, query_balance, query_borrower_info, query_token_balance,
    BorrowerInfoResponse,
};

pub fn update_global_reward(
    deps: Deps,
    env: Env,
    config: &Config,
    state: &mut State,
) -> Result<(), ContractError> {
    if state.last_reward_updated >= env.block.height {
        return Ok(());
    }

    // 1. get amount of borrowed UST
    // 2. get amount of aUST
    // 3. get aUST to UST ratio
    // 4. get amount of UST
    // 5. (aUST_amount * aUST_ration) + UST_amount - borrowed_ust = rewards

    let borrower_info: BorrowerInfoResponse = query_borrower_info(
        deps,
        &config.anchor_market_contract,
        &config.basset_farmer_contract,
    )?;
    let borrowed_amount = borrower_info.loan_amount;

    let aterra_balance =
        query_token_balance(deps, &config.aterra_token, &config.basset_farmer_contract)?;
    let casset_staked_amount =
        query_token_balance(deps, &config.casset_token, &env.contract.address)?;
    let stable_coin_balance = query_balance(
        &deps.querier,
        &config.basset_farmer_contract,
        config.stable_denom.clone(),
    )?;

    let aterra_state = query_aterra_state(deps, &config.anchor_market_contract)?;

    calculate_reward_index(
        state,
        borrowed_amount,
        aterra_balance.into(),
        aterra_state.exchange_rate,
        stable_coin_balance.into(),
        casset_staked_amount.into(),
    );

    state.last_reward_updated = env.block.height;

    Ok(())
}

//TODO: this method is wrong
//test all cases where you are using it!
pub fn update_staker_reward(state: &State, staker_state: &mut StakerState) {
    let currently_staked = Decimal256::from_uint256(staker_state.staked_amount);
    staker_state.pending_rewards +=
        (state.global_reward_index - staker_state.reward_index) * currently_staked;
    staker_state.reward_index = state.global_reward_index;
}

fn calculate_reward_index(
    state: &mut State,
    borrowed_amount: Uint256,
    aterra_amount: Uint256,
    aterra_exchange_rate: Decimal256,
    stable_coin_amount: Uint256,
    casset_staked_amount: Uint256,
) {
    //TODO: remove me
    println!("borrowed_amount: {}, aterra_amount: {}, aterra_exchange_rate: {}, stable_coin_amount: {}, casset_staked_amount: {}", 
    borrowed_amount,
    aterra_amount,
    aterra_exchange_rate,
    stable_coin_amount,
    casset_staked_amount,
);
    let stable_balance = aterra_amount * aterra_exchange_rate + stable_coin_amount;
    let decimal_casset_staked_amount = Decimal256::from_uint256(casset_staked_amount);

    if borrowed_amount >= stable_balance {
        return;
    } else {
        let current_total_reward_amount = stable_balance - borrowed_amount;
        let current_total_reward_amount: Decimal256 =
            Decimal256::from_uint256(current_total_reward_amount);

        if current_total_reward_amount < state.last_reward_amount {
            return;
        } else {
            let new_reward_amount: Decimal256 =
                current_total_reward_amount - state.last_reward_amount;
            println!("new_reward_amount: {}", new_reward_amount);
            println!(
                "decimal_casset_staked_amount: {}",
                decimal_casset_staked_amount
            );

            state.global_reward_index += new_reward_amount / decimal_casset_staked_amount;
        }
        state.last_reward_amount = current_total_reward_amount;
    }

    //TODO: remove me
    println!("state after calc_reward_index: {:?}", state);
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
            last_reward_updated: 0u64,
        };
        let borrowed_amount = Uint256::from(0u64);
        let aterra_amount = Uint256::from(0u64);
        let aterra_exchange_rate = Decimal256::zero();
        let ust_amount = Uint256::from(0u64);
        let casset_staked_amount = Uint256::from(0u64);

        calculate_reward_index(
            &mut state,
            borrowed_amount,
            aterra_amount,
            aterra_exchange_rate,
            ust_amount,
            casset_staked_amount,
        );
        assert_eq!(state.last_reward_amount, Decimal256::zero());
        assert_eq!(state.global_reward_index, Decimal256::zero());
    }

    #[test]
    fn reward_calc_first_reward() {
        let mut state = State {
            global_reward_index: Decimal256::zero(),
            last_reward_amount: Decimal256::zero(),
            last_reward_updated: 0u64,
        };
        let borrowed_amount = Uint256::from(200_000u64);
        let aterra_amount = Uint256::from(195_000u64);
        let aterra_exchange_rate = Decimal256::from_str("1.1").unwrap();
        let ust_amount = Uint256::from(10_000u64);
        let casset_staked_amount = Uint256::from(100u64);
        //195 * 1.1 = 214.5; 214.5 + 10 - 200 = 24.5

        calculate_reward_index(
            &mut state,
            borrowed_amount,
            aterra_amount,
            aterra_exchange_rate,
            ust_amount,
            casset_staked_amount,
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
            last_reward_updated: 0u64,
        };
        let borrowed_amount = Uint256::from(300_000u64);
        let aterra_amount = Uint256::from(290_000u64);
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let ust_amount = Uint256::from(10_000u64);
        let casset_staked_amount = Uint256::from(100u64);
        //290 * 1.2 = 348; 348 + 10 - 300 = 58

        let prev_global_reward_index = state.global_reward_index;
        calculate_reward_index(
            &mut state,
            borrowed_amount,
            aterra_amount,
            aterra_exchange_rate,
            ust_amount,
            casset_staked_amount,
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
            last_reward_updated: 0u64,
        };
        let borrowed_amount = Uint256::from(300_000u64);
        let aterra_amount = Uint256::from(200_000u64);
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let ust_amount = Uint256::from(10_000u64);
        let casset_staked_amount = Uint256::from(100u64);
        //200 * 1.2 = 240; 240 + 10 - 300 = -50
        //so borrowed_amount is less than UST balance, means do not change rewards

        calculate_reward_index(
            &mut state,
            borrowed_amount,
            aterra_amount,
            aterra_exchange_rate,
            ust_amount,
            casset_staked_amount,
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
            last_reward_updated: 0u64,
        };
        let borrowed_amount = Uint256::from(300_000u64);
        let aterra_amount = Uint256::from(275_000u64);
        let aterra_exchange_rate = Decimal256::from_str("1.1").unwrap();
        let ust_amount = Uint256::from(10_000u64);
        let casset_staked_amount = Uint256::from(100u64);
        //275 * 1.1 = 302.5; 302.5 + 10 - 300 = 12.5
        //12.5 is less then previous reward amount, means do not change rewards

        calculate_reward_index(
            &mut state,
            borrowed_amount,
            aterra_amount,
            aterra_exchange_rate,
            ust_amount,
            casset_staked_amount,
        );
        assert_eq!(state.last_reward_amount, last_reward_amount);
        assert_eq!(state.global_reward_index, global_reward_index);
    }
}
