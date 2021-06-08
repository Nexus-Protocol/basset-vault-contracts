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

pub fn calculate_reward_index(deps: DepsMut, env: Env, state: &mut State) -> StdResult<()> {
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

    let aust_balance =
        query_token_balance(deps.as_ref(), &config.aterra_token, &env.contract.address)?;

    let aust_state = query_aterra_state(deps.as_ref(), &config.anchor_market_contract)?;

    let ust_balance = Uint256::from(aust_balance) * aust_state.exchange_rate;
    let decimal_ust_balance = Decimal256::from_uint256(ust_balance);
    let borrowed_ust = borrower_info.loan_amount;

    if borrowed_ust >= ust_balance {
        state.global_reward_index = Decimal256::zero();
        state.last_reward_amount = Decimal256::zero();
    } else {
        let current_total_reward_amount = ust_balance - borrowed_ust;
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

    Ok(())
}
