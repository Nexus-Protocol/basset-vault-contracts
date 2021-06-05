use cosmwasm_std::{Deps, StdResult};
use yield_optimizer::basset_farmer::ConfigResponse;

use crate::state::load_config;
use crate::state::Config;

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = load_config(deps.storage)?;
    Ok(ConfigResponse {
        overseer_contract: config.overseer_contract.to_string(),
        custody_basset_contract: config.custody_basset_contract.to_string(),
        casset_token: config.casset_token.to_string(),
        basset_token: config.basset_token.to_string(),
    })
}
