use cosmwasm_std::{Deps, StdResult};
use yield_optimizer::basset_farmer_config::{BorrowLimitsResponse, ConfigResponse, StateResponse};

use crate::state::{load_config, load_state, Config};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = load_config(deps.storage)?;
    Ok(ConfigResponse {
        borrow_ration_aim: config.borrow_ration_aim,
        borrow_ration_upper_gap: config.borrow_ration_upper_gap,
        borrow_ration_bottom_gap: config.borrow_ration_bottom_gap,
        oracle_addr: config.oracle_addr.to_string(),
        basset_token_addr: config.basset_token_addr.to_string(),
        stable_denom: config.stable_denom,
    })
}

pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = load_state(deps.storage)?;
    Ok(StateResponse {
        prices: state.prices,
        price_last_update_time: state.price_last_update_time,
        last_std_dev_from_average_price: state.last_std_dev_from_average_price,
    })
}

pub fn borrow_limits() -> StdResult<BorrowLimitsResponse> {
    todo!()
}
