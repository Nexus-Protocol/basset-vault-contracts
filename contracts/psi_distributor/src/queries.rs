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
        .map(|dis| (dis.recipient.to_string(), dis.share))
        .collect();

    Ok(ConfigResponse {
        psi_token_addr: config.psi_token.to_string(),
        governance_contract_addr: config.governance_contract.to_string(),
        rewards_distribution,
    })
}
