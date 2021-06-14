use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Addr, Binary, CanonicalAddr, CosmosMsg, Deps,
    DepsMut, Env, MessageInfo, Reply, ReplyOn, Response, StdError, StdResult, SubMsg, Uint128,
    WasmMsg,
};
use terraswap::querier::{query_balance, query_supply};

use crate::state::{load_config, Config, State};
use yield_optimizer::{
    querier::{
        query_aterra_state, query_borrower_info, query_token_balance, AnchorMarketMsg,
        BorrowerInfoResponse,
    },
    subtract_tax, TaxInfo,
};

pub fn update_reward_index(deps: DepsMut, env: Env, state: &mut State) -> StdResult<()> {
    //TODO: provide config from outside?!
    let config: Config = load_config(deps.storage)?;

    // 1. get amount of borrowed UST
    // 2. get amount of aUST you have
    // 3. get aUST to UST ratio
    // 4. get amount of UST you have
    // 5. get cAsset total supply
    // 4. (aUST_amount * aUST_ration) + UST_amount - borrowed_ust = rewards

    let borrower_info: BorrowerInfoResponse = query_borrower_info(
        deps.as_ref(),
        &config.anchor_market_contract,
        &env.contract.address,
    )?;
    let borrowed_ust = borrower_info.loan_amount;

    //TODO: try to store UST, aUST balances and cAsset supply in State - too many queries here
    let aust_balance =
        query_token_balance(deps.as_ref(), &config.aterra_token, &env.contract.address)?;
    //TODO: do not count entire supply. Count tokens in staking!
    let casset_token_supply = query_supply(&deps.querier, config.casset_token)?;
    let ust_balance = query_balance(&deps.querier, env.contract.address, "uust".to_string())?;

    let aust_state = query_aterra_state(deps.as_ref(), &config.anchor_market_contract)?;

    calculate_reward_index(
        state,
        borrowed_ust,
        Uint256::from(aust_balance),
        aust_state.exchange_rate,
        Uint256::from(ust_balance),
        Uint256::from(casset_token_supply),
    );
    //TODO: maybe save updated state here???!!!
    Ok(())
}

//TODO: on user reward calculation check if his index > curren_index. If true, then do nothing and
//wait for reward index to be bigger.
//TOOD: what to do if user want to Withdraw, but his reward_index is bigger then current?!
//I think good choice is to cut some of his bAsset, based on price.
fn calculate_reward_index(
    state: &mut State,
    borrowed_amount: Uint256,
    aterra_amount: Uint256,
    aterra_exchange_rate: Decimal256,
    ust_amount: Uint256,
    casset_token_supply: Uint256,
) {
    let ust_balance = aterra_amount * aterra_exchange_rate + ust_amount;
    let decimal_casset_token_supply = Decimal256::from_uint256(casset_token_supply);

    if borrowed_amount >= ust_balance {
        state.global_reward_index = Decimal256::zero();
        state.last_reward_amount = Decimal256::zero();
    } else {
        let current_total_reward_amount = ust_balance - borrowed_amount;
        let current_total_reward_amount: Decimal256 =
            Decimal256::from_uint256(current_total_reward_amount);

        if current_total_reward_amount < state.last_reward_amount {
            let new_reward_amount_negative: Decimal256 =
                state.last_reward_amount - current_total_reward_amount;
            state.global_reward_index = state.global_reward_index
                - (new_reward_amount_negative / decimal_casset_token_supply);
        } else {
            let new_reward_amount: Decimal256 =
                current_total_reward_amount - state.last_reward_amount;
            state.global_reward_index += new_reward_amount / decimal_casset_token_supply;
        }
        state.last_reward_amount = current_total_reward_amount;
    }
}

pub fn calculate_aterra_amount_to_sell(
    aterra_balance: Uint256,
    stable_coin_balance: Uint256,
    aterra_exchange_rate: Decimal256,
    repay_amount: Uint256,
    aim_buffer_size: Uint256,
) -> Uint256 {
    if aterra_exchange_rate == Decimal256::zero() {
        //no reason to sell
        return Uint256::zero();
    }

    let current_total_stable_balance = aterra_balance * aterra_exchange_rate + stable_coin_balance;

    if repay_amount > current_total_stable_balance {
        //if we need to repay more than we have - means we farming with loss
        //- repay as much as can
        //TODO: looks like we can't withdraw all users bAsset. Ignore that case for the moment
        aterra_balance
    } else if (current_total_stable_balance - repay_amount) <= aim_buffer_size {
        //means we need to sell all aterra and use rest as a buffer
        aterra_balance
    } else {
        //means we need to repay only by selling some aterra
        //OR just get it from buffer without selling aterra

        if stable_coin_balance > aim_buffer_size {
            let take_from_buffer = stable_coin_balance - aim_buffer_size;
            if take_from_buffer >= repay_amount {
                //repay from buffer, no need to sell aterra
                Uint256::zero()
            } else {
                //sell diff between between amount that you can take from buffer and 'repay_amount'
                let aterra_to_repay = (repay_amount - take_from_buffer) / aterra_exchange_rate;
                aterra_to_repay
            }
        } else {
            //sell aterra to repay loan AND to fill the buffer
            let add_to_buffer = aim_buffer_size - stable_coin_balance;
            let aterra_to_repay = (repay_amount + add_to_buffer) / aterra_exchange_rate;
            aterra_to_repay
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct AterraRedeemErrorHandlingAction {
    pub repay_loan_amount: Uint256,
    pub aterra_amount_to_sell: Uint256,
}

//TODO: тут должна учитываться инфа о том сколько надо долга выплатить!!!
pub fn calc_aterra_redeem_error_handling_action(
    aterra_amount_to_sell: Uint256,
    aterra_exchange_rate: Decimal256,
    current_buffer_balance: Uint256,
    aim_buffer_size: Uint256,
    tax_info: &TaxInfo,
) -> Option<AterraRedeemErrorHandlingAction> {
    if aterra_amount_to_sell == Uint256::zero() {
        if current_buffer_balance > aim_buffer_size {
            //we do not need to sell aterra anymore, so repay remainder from buffer
            let loan_to_repay = current_buffer_balance - aim_buffer_size;
            let repaying_loan_size_after_tax = subtract_tax(loan_to_repay, tax_info);
            let result = AterraRedeemErrorHandlingAction {
                repay_loan_amount: repaying_loan_size_after_tax,
                aterra_amount_to_sell,
            };
            return Some(result);
        } else {
            return None;
        }
    }

    let aterra_value = aterra_amount_to_sell * aterra_exchange_rate;
    if aterra_value >= current_buffer_balance {
        //sell entire buffer, because we need to redeem more aterra
        let loan_to_repay = current_buffer_balance;
        let repaying_loan_size_after_tax = subtract_tax(loan_to_repay, tax_info);
        let result = AterraRedeemErrorHandlingAction {
            repay_loan_amount: repaying_loan_size_after_tax,
            aterra_amount_to_sell: repaying_loan_size_after_tax / aterra_exchange_rate,
        };
        return Some(result);
    } else {
        //sell last part of aterra
        let new_buffer_balance = current_buffer_balance + aterra_value;
        let repaying_loan_size_after_tax = if new_buffer_balance > aim_buffer_size {
            subtract_tax(new_buffer_balance - aim_buffer_size, tax_info)
        } else {
            Uint256::zero()
        };
        let result = AterraRedeemErrorHandlingAction {
            repay_loan_amount: repaying_loan_size_after_tax,
            aterra_amount_to_sell,
        };
        return Some(result);
    }
}

#[cfg(test)]
mod test {

    use super::{
        calc_aterra_redeem_error_handling_action, calculate_aterra_amount_to_sell,
        calculate_reward_index, AterraRedeemErrorHandlingAction,
    };
    use crate::state::State;
    use cosmwasm_bignumber::{Decimal256, Uint256};
    use std::str::FromStr;
    use yield_optimizer::TaxInfo;

    #[test]
    fn reward_calc_zero_state() {
        let mut state = State {
            global_reward_index: Decimal256::zero(),
            last_reward_amount: Decimal256::zero(),
        };
        let borrowed_amount = Uint256::from(0u64);
        let aterra_amount = Uint256::from(0u64);
        let aterra_exchange_rate = Decimal256::zero();
        let ust_amount = Uint256::from(0u64);
        let casset_token_supply = Uint256::from(0u64);

        calculate_reward_index(
            &mut state,
            borrowed_amount,
            aterra_amount,
            aterra_exchange_rate,
            ust_amount,
            casset_token_supply,
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
        let ust_amount = Uint256::from(10_000u64);
        let casset_token_supply = Uint256::from(100u64);
        //195 * 1.1 = 214.5; 214.5 + 10 - 200 = 24.5

        calculate_reward_index(
            &mut state,
            borrowed_amount,
            aterra_amount,
            aterra_exchange_rate,
            ust_amount,
            casset_token_supply,
        );
        assert_eq!(
            state.last_reward_amount,
            Decimal256::from_str("24500").unwrap()
        );
        // 24_500 / 100
        assert_eq!(
            state.global_reward_index,
            Decimal256::from_ratio(Uint256::from(24_500u64).0, Uint256::from(100u64).0)
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
        };
        let borrowed_amount = Uint256::from(300_000u64);
        let aterra_amount = Uint256::from(290_000u64);
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let ust_amount = Uint256::from(10_000u64);
        let casset_token_supply = Uint256::from(100u64);
        //290 * 1.2 = 348; 348 + 10 - 300 = 58

        let prev_global_reward_index = state.global_reward_index;
        calculate_reward_index(
            &mut state,
            borrowed_amount,
            aterra_amount,
            aterra_exchange_rate,
            ust_amount,
            casset_token_supply,
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
        //get from 'first reward' test
        let mut state = State {
            global_reward_index: Decimal256::from_ratio(
                Uint256::from(24_500u64).0,
                Uint256::from(100u64).0,
            ),
            last_reward_amount: Decimal256::from_str("14500").unwrap(),
        };
        let borrowed_amount = Uint256::from(300_000u64);
        let aterra_amount = Uint256::from(200_000u64);
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let ust_amount = Uint256::from(10_000u64);
        let casset_token_supply = Uint256::from(100u64);
        //200 * 1.2 = 240; 240 + 10 - 300 = -50
        //so borrowed_amount is less than UST balance, means drop all rewards to zero

        calculate_reward_index(
            &mut state,
            borrowed_amount,
            aterra_amount,
            aterra_exchange_rate,
            ust_amount,
            casset_token_supply,
        );
        assert_eq!(state.last_reward_amount, Decimal256::zero());
        assert_eq!(state.global_reward_index, Decimal256::zero());
    }

    #[test]
    fn reward_calc_negative_reward() {
        //get from 'first reward' test
        let mut state = State {
            global_reward_index: Decimal256::from_ratio(
                Uint256::from(24_500u64).0,
                Uint256::from(100u64).0,
            ),
            last_reward_amount: Decimal256::from_str("14500").unwrap(),
        };
        let borrowed_amount = Uint256::from(300_000u64);
        let aterra_amount = Uint256::from(275_000u64);
        let aterra_exchange_rate = Decimal256::from_str("1.1").unwrap();
        let ust_amount = Uint256::from(10_000u64);
        let casset_token_supply = Uint256::from(100u64);
        //275 * 1.1 = 302.5; 302.5 + 10 - 300 = 12.5
        //12.5 is less then previous reward amount, means we need to deduct reward_index

        let prev_global_reward_index = state.global_reward_index;
        calculate_reward_index(
            &mut state,
            borrowed_amount,
            aterra_amount,
            aterra_exchange_rate,
            ust_amount,
            casset_token_supply,
        );
        assert_eq!(
            state.last_reward_amount,
            Decimal256::from_str("12500").unwrap()
        );
        // 12_500 - 14_500 = -2_000 (new_reward)
        assert_eq!(
            state.global_reward_index,
            prev_global_reward_index
                - Decimal256::from_ratio(Uint256::from(2_000u64).0, Uint256::from(100u64).0)
        );
    }

    #[test]
    fn aterra_sell_calc_sell_all_1() {
        let aterra_balance = Uint256::from(500u64);
        let stable_coin_balance = Uint256::from(100u64);
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let repay_amount = Uint256::from(1_000u64);
        let aim_buffer_size = Uint256::from(100u64);
        let aterra_to_sell = calculate_aterra_amount_to_sell(
            aterra_balance,
            stable_coin_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
        );
        assert_eq!(aterra_balance, aterra_to_sell);
    }

    #[test]
    fn aterra_sell_calc_sell_all_2() {
        let aterra_balance = Uint256::zero();
        let stable_coin_balance = Uint256::zero();
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let repay_amount = Uint256::from(1_000u64);
        let aim_buffer_size = Uint256::from(100u64);
        let aterra_to_sell = calculate_aterra_amount_to_sell(
            aterra_balance,
            stable_coin_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
        );
        assert_eq!(aterra_balance, aterra_to_sell);
    }

    #[test]
    fn aterra_sell_calc_sell_all_3() {
        let aterra_balance = Uint256::zero();
        let stable_coin_balance = Uint256::from(200u64);
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let repay_amount = Uint256::from(1_000u64);
        let aim_buffer_size = Uint256::from(100u64);
        let aterra_to_sell = calculate_aterra_amount_to_sell(
            aterra_balance,
            stable_coin_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
        );
        assert_eq!(aterra_balance, aterra_to_sell);
    }

    #[test]
    fn aterra_sell_calc_sell_all_4() {
        let aterra_balance = Uint256::from(200u64);
        let stable_coin_balance = Uint256::zero();
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let repay_amount = Uint256::from(1_000u64);
        let aim_buffer_size = Uint256::from(100u64);
        let aterra_to_sell = calculate_aterra_amount_to_sell(
            aterra_balance,
            stable_coin_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
        );
        assert_eq!(aterra_balance, aterra_to_sell);
    }

    #[test]
    fn aterra_sell_calc_fill_aim_buffer() {
        let aterra_balance = Uint256::from(100u64);
        let stable_coin_balance = Uint256::from(100u64);
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let repay_amount = Uint256::from(200u64);
        //total: 100+120
        //total - repay = 220 - 200 = 20 which is less then we need to aim_buffer_size, so sell all
        let aim_buffer_size = Uint256::from(100u64);
        let aterra_to_sell = calculate_aterra_amount_to_sell(
            aterra_balance,
            stable_coin_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
        );
        assert_eq!(aterra_balance, aterra_to_sell);
    }

    #[test]
    fn aterra_sell_calc_no_need_to_sell() {
        let aterra_balance = Uint256::from(100u64);
        let stable_coin_balance = Uint256::from(100u64);
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let repay_amount = Uint256::from(50u64);
        //ust_buffer >= repay + aim_buffer
        //so no need to sell
        let aim_buffer_size = Uint256::from(50u64);
        let aterra_to_sell = calculate_aterra_amount_to_sell(
            aterra_balance,
            stable_coin_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
        );
        assert_eq!(Uint256::zero(), aterra_to_sell);
    }

    #[test]
    fn aterra_sell_calc_get_portion_from_buffer() {
        let aterra_balance = Uint256::from(100u64);
        let stable_coin_balance = Uint256::from(100u64);
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let repay_amount = Uint256::from(80u64);
        //we can get 20 coins from buffer, cause aim_buffer < current_buffer
        //need to sell only to get 80 - 20 = 60 coins
        let aim_buffer_size = Uint256::from(80u64);
        let aterra_to_sell = calculate_aterra_amount_to_sell(
            aterra_balance,
            stable_coin_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
        );
        assert_eq!(Uint256::from(50u64), aterra_to_sell);
    }

    #[test]
    fn aterra_sell_calc_sell_to_add_to_buffer() {
        let aterra_balance = Uint256::from(100u64);
        let stable_coin_balance = Uint256::from(100u64);
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let repay_amount = Uint256::from(70u64);
        //we need to add 20 coins to buffer, cause aim_buffer > current_buffer
        //need to sell to get 70 + 20 = 90 coins
        let aim_buffer_size = Uint256::from(120u64);
        let aterra_to_sell = calculate_aterra_amount_to_sell(
            aterra_balance,
            stable_coin_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
        );
        assert_eq!(Uint256::from(75u64), aterra_to_sell);
    }

    #[test]
    fn aterra_redeem_error_handling_action_do_nothing() {
        let aterra_amount_to_sell = Uint256::zero();
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let current_buffer_balance = Uint256::from(100u64);
        let aim_buffer_size = Uint256::from(100u64);
        let tax_info = TaxInfo {
            rate: Decimal256::from_str("0.01").unwrap(),
            cap: Uint256::from(15u64),
        };
        let error_handling_action = calc_aterra_redeem_error_handling_action(
            aterra_amount_to_sell,
            aterra_exchange_rate,
            current_buffer_balance,
            aim_buffer_size,
            &tax_info,
        );
        assert!(error_handling_action.is_none());
    }

    #[test]
    fn aterra_redeem_error_handling_action_repay_from_buffer() {
        let aterra_amount_to_sell = Uint256::zero();
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let current_buffer_balance = Uint256::from(100_000u64);
        let aim_buffer_size = Uint256::from(80_000u64);
        let tax_info = TaxInfo {
            rate: Decimal256::from_str("0.6").unwrap(),
            cap: Uint256::from(99999999u64),
        };
        let error_handling_action = calc_aterra_redeem_error_handling_action(
            aterra_amount_to_sell,
            aterra_exchange_rate,
            current_buffer_balance,
            aim_buffer_size,
            &tax_info,
        );
        let expected_action = AterraRedeemErrorHandlingAction {
            repay_loan_amount: Uint256::from(12_500u64), //0.6 tax means: 12_500 * 0.6 + 12_500 = 20_000
            aterra_amount_to_sell: Uint256::zero(),
        };

        assert_eq!(expected_action, error_handling_action.unwrap());
    }

    #[test]
    fn aterra_redeem_error_handling_action_sell_entire_buffer() {
        let aterra_amount_to_sell = Uint256::from(300_000u64);
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let current_buffer_balance = Uint256::from(100_000u64);
        let aim_buffer_size = Uint256::from(80_000u64);
        let tax_info = TaxInfo {
            rate: Decimal256::from_str("0.01").unwrap(),
            cap: Uint256::from(750u64),
        };
        let error_handling_action = calc_aterra_redeem_error_handling_action(
            aterra_amount_to_sell,
            aterra_exchange_rate,
            current_buffer_balance,
            aim_buffer_size,
            &tax_info,
        );
        let expected_action = AterraRedeemErrorHandlingAction {
            repay_loan_amount: Uint256::from(99_250u64), //minus 750 tax cap
            aterra_amount_to_sell: Uint256::from(99_250u64) / aterra_exchange_rate,
        };

        assert_eq!(expected_action, error_handling_action.unwrap());
    }

    #[test]
    fn aterra_redeem_error_handling_action_last_aterra_sell() {
        let aterra_amount_to_sell = Uint256::from(20_000u64);
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let current_buffer_balance = Uint256::from(100_000u64);
        let aim_buffer_size = Uint256::from(80_000u64);
        let tax_info = TaxInfo {
            rate: Decimal256::from_str("0.02").unwrap(),
            cap: Uint256::from(750u64),
        };
        let error_handling_action = calc_aterra_redeem_error_handling_action(
            aterra_amount_to_sell,
            aterra_exchange_rate,
            current_buffer_balance,
            aim_buffer_size,
            &tax_info,
        );
        //24_000 get from selling aterra
        //so, repay amount should be: 100_000 - 80_000 + 24_000 = 44_000
        //tax: 44_000 * (1 - 1/(1+0.02)) ~= 862 which is bigger than cap
        let expected_action = AterraRedeemErrorHandlingAction {
            repay_loan_amount: Uint256::from(43_250u64), //minus 750 tax cap
            aterra_amount_to_sell,
        };

        assert_eq!(expected_action, error_handling_action.unwrap());
    }

    #[test]
    fn aterra_redeem_error_handling_action_last_aterra_sell_but_without_loan_repaying() {
        let aterra_amount_to_sell = Uint256::from(20_000u64);
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let current_buffer_balance = Uint256::from(40_000u64);
        let aim_buffer_size = Uint256::from(80_000u64);
        let tax_info = TaxInfo {
            rate: Decimal256::from_str("0.01").unwrap(),
            cap: Uint256::from(750u64),
        };
        let error_handling_action = calc_aterra_redeem_error_handling_action(
            aterra_amount_to_sell,
            aterra_exchange_rate,
            current_buffer_balance,
            aim_buffer_size,
            &tax_info,
        );
        //24_000 get from selling aterra
        //so, repay amount should be: 40_000 - 80_000 + 24_000 = -16_000
        //no need to repaying loan
        let expected_action = AterraRedeemErrorHandlingAction {
            repay_loan_amount: Uint256::zero(),
            aterra_amount_to_sell,
        };

        assert_eq!(expected_action, error_handling_action.unwrap());
    }
}
