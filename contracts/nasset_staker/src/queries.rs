use cosmwasm_std::{Deps, StdResult};
use yield_optimizer::nasset_staker::ConfigResponse;

use crate::state::load_config;
use crate::state::Config;

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = load_config(deps.storage)?;
    Ok(ConfigResponse {
        nasset_token: config.nasset_token.to_string(),
    })
}
