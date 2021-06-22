use cosmwasm_std::{Deps, StdResult};
use yield_optimizer::casset_staking::ConfigResponse;

use crate::state::load_config;
use crate::state::Config;

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = load_config(deps.storage)?;
    Ok(ConfigResponse {
        casset_token: config.casset_token.to_string(),
        aterra_token: config.aterra_token.to_string(),
        stable_denom: config.stable_denom,
        basset_farmer_contract: config.basset_farmer_contract.to_string(),
        anchor_market_contract: config.anchor_market_contract.to_string(),
    })
}
