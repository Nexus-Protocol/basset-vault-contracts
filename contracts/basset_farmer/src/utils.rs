use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Addr, Binary, CanonicalAddr, CosmosMsg, Deps,
    DepsMut, Env, MessageInfo, Reply, ReplyOn, Response, StdError, StdResult, SubMsg, Uint128,
    WasmMsg,
};

use crate::state::{load_config, Config, State};
use yield_optimizer::querier::{
    query_aterra_state, query_borrower_info, query_token_balance, AnchorMarketMsg,
    BorrowerInfoResponse,
};

pub fn update_reward_index(deps: DepsMut, env: Env, state: &mut State) -> StdResult<()> {
    //TODO: provide config from outside?!
    let config: Config = load_config(deps.storage)?;

    // 1. get amount of borrowed UST
    // 2. get amount of aUST you have
    // 3. get aUST to UST ratio
    // 4. (aUST_amount * aUST_ration) - borrowed_ust = rewards

    let borrower_info: BorrowerInfoResponse = query_borrower_info(
        deps.as_ref(),
        &config.anchor_market_contract,
        &env.contract.address,
    )?;
    let borrowed_ust = borrower_info.loan_amount;

    let aust_balance =
        query_token_balance(deps.as_ref(), &config.aterra_token, &env.contract.address)?;

    let aust_state = query_aterra_state(deps.as_ref(), &config.anchor_market_contract)?;

    calculate_reward_index(
        state,
        borrowed_ust,
        Uint256::from(aust_balance),
        aust_state.exchange_rate,
    );
    Ok(())
}

fn calculate_reward_index(
    state: &mut State,
    borrowed_amount: Uint256,
    aterra_amount: Uint256,
    aterra_exchange_rate: Decimal256,
) {
    let ust_balance = aterra_amount * aterra_exchange_rate;
    let decimal_ust_balance = Decimal256::from_uint256(ust_balance);

    if borrowed_amount >= ust_balance {
        state.global_reward_index = Decimal256::zero();
        state.last_reward_amount = Decimal256::zero();
    } else {
        let current_total_reward_amount = ust_balance - borrowed_amount;
        let current_total_reward_amount: Decimal256 =
            Decimal256::from_uint256(current_total_reward_amount);

        if current_total_reward_amount < state.last_reward_amount {
            //TODO: negative amount of reward
            //TODO: not sure about this - write test!
            let new_reward_amount_negative: Decimal256 =
                state.last_reward_amount - current_total_reward_amount;
            state.global_reward_index =
                state.global_reward_index - (new_reward_amount_negative / decimal_ust_balance);
        } else {
            let new_reward_amount: Decimal256 =
                current_total_reward_amount - state.last_reward_amount;
            state.global_reward_index += new_reward_amount / decimal_ust_balance;
        }
        state.last_reward_amount = current_total_reward_amount;
    }
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
        };
        let borrowed_amount = Uint256::from(0u64);
        let aterra_amount = Uint256::from(0u64);
        let aterra_exchange_rate = Decimal256::zero();

        calculate_reward_index(
            &mut state,
            borrowed_amount,
            aterra_amount,
            aterra_exchange_rate,
        );
        assert_eq!(state.last_reward_amount, Decimal256::zero());
        assert_eq!(state.global_reward_index, Decimal256::zero());
    }

    #[test]
    fn reward_calc_first_reward() {
        let mut state = State {
            global_reward_index: Decimal256::zero(),
            last_reward_amount: Decimal256::zero(),
        };
        let borrowed_amount = Uint256::from(200_000u64);
        let aterra_amount = Uint256::from(195_000u64);
        let aterra_exchange_rate = Decimal256::from_str("1.1").unwrap();
        //195 * 1.1 = 214.5; 214.5 - 200 = 14.5

        calculate_reward_index(
            &mut state,
            borrowed_amount,
            aterra_amount,
            aterra_exchange_rate,
        );
        assert_eq!(
            state.last_reward_amount,
            Decimal256::from_str("14500").unwrap()
        );
        // 14_500 / 214_500
        assert_eq!(
            state.global_reward_index,
            Decimal256::from_ratio(Uint256::from(14_500u64).0, Uint256::from(214_500u64).0)
        );
    }

    #[test]
    fn reward_calc_second_reward() {
        //get from 'first reward' test
        let mut state = State {
            global_reward_index: Decimal256::from_ratio(
                Uint256::from(14_500u64).0,
                Uint256::from(214_500u64).0,
            ),
            last_reward_amount: Decimal256::from_str("14500").unwrap(),
        };
        let borrowed_amount = Uint256::from(300_000u64);
        let aterra_amount = Uint256::from(290_000u64);
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        //290 * 1.2 = 348; 348 - 300 = 48

        let prev_global_reward_index = state.global_reward_index;
        calculate_reward_index(
            &mut state,
            borrowed_amount,
            aterra_amount,
            aterra_exchange_rate,
        );
        assert_eq!(
            state.last_reward_amount,
            Decimal256::from_str("48000").unwrap()
        );
        // new_rewards = 48_000 - 14_500 = 33_500
        assert_eq!(
            state.global_reward_index,
            prev_global_reward_index
                + Decimal256::from_ratio(Uint256::from(33_500u64).0, Uint256::from(348_000u64).0)
        );
    }

    #[test]
    fn reward_calc_borrowed_amount_bigger_then_aterra_amount() {
        //get from 'first reward' test
        let mut state = State {
            global_reward_index: Decimal256::from_ratio(
                Uint256::from(14_500u64).0,
                Uint256::from(214_500u64).0,
            ),
            last_reward_amount: Decimal256::from_str("14500").unwrap(),
        };
        let borrowed_amount = Uint256::from(300_000u64);
        let aterra_amount = Uint256::from(200_000u64);
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        //200 * 1.2 = 240; 240 - 300 = -60
        //so borrowed_amount is less than UST balance, means drop all rewards to zero

        calculate_reward_index(
            &mut state,
            borrowed_amount,
            aterra_amount,
            aterra_exchange_rate,
        );
        assert_eq!(state.last_reward_amount, Decimal256::zero());
        assert_eq!(state.global_reward_index, Decimal256::zero());
    }

    #[test]
    fn reward_calc_negative_reward() {
        //get from 'first reward' test
        let mut state = State {
            global_reward_index: Decimal256::from_ratio(
                Uint256::from(14_500u64).0,
                Uint256::from(214_500u64).0,
            ),
            last_reward_amount: Decimal256::from_str("14500").unwrap(),
        };
        let borrowed_amount = Uint256::from(300_000u64);
        let aterra_amount = Uint256::from(280_000u64);
        let aterra_exchange_rate = Decimal256::from_str("1.1").unwrap();
        //280 * 1.1 = 308; 308 - 300 = 8
        //8 is less then previous reward amount, means we need to deduct reward_index

        let prev_global_reward_index = state.global_reward_index;
        calculate_reward_index(
            &mut state,
            borrowed_amount,
            aterra_amount,
            aterra_exchange_rate,
        );
        assert_eq!(
            state.last_reward_amount,
            Decimal256::from_str("8000").unwrap()
        );
        // 8000 - 14_500 = -6_500 (new_reward)
        assert_eq!(
            state.global_reward_index,
            prev_global_reward_index
                - Decimal256::from_ratio(Uint256::from(6_500u64).0, Uint256::from(308_000u64).0)
        );
    }
}
