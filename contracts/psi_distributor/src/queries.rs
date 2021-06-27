use cosmwasm_std::{Deps, StdResult};
use yield_optimizer::psi_distributor::ConfigResponse;

use crate::state::load_config;
use crate::state::Config;

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = load_config(deps.storage)?;
    let rewards_distribution = config
        .rewards_distribution
        .distribution()
        .iter()
        .map(|dis| (dis.recepient.to_string(), dis.share))
        .collect();

    Ok(ConfigResponse {
        nasset_token_addr: config.nasset_token_addr.to_string(),
        governance_addr: config.governance_addr.to_string(),
        rewards_distribution,
    })
}
