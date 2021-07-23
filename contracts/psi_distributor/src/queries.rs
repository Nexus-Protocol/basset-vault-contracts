use basset_vault::psi_distributor::ConfigResponse;
use cosmwasm_std::{Deps, StdResult};

use crate::state::load_config;
use crate::state::Config;

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = load_config(deps.storage)?;

    Ok(ConfigResponse {
        psi_token_addr: config.psi_token.to_string(),
        governance_contract_addr: config.governance_contract.to_string(),
        nasset_token_rewards_contract_addr: config.nasset_token_rewards_contract.to_string(),
        community_pool_contract_addr: config.community_pool_contract.to_string(),
        basset_vault_strategy_contract_addr: config.basset_vault_strategy_contract.to_string(),
        manual_ltv: config.manual_ltv,
        fee_rate: config.fee_rate,
        tax_rate: config.tax_rate,
    })
}
