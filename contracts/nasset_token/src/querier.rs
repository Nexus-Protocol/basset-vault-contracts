use cosmwasm_std::{Addr, Binary, Deps, QueryRequest, StdResult, WasmQuery};

use crate::state::load_config_holder_contract;
use basset_vault::nasset_token_config_holder::Config;

pub fn query_rewards_contract(deps: Deps) -> StdResult<Addr> {
    let config_holder_contract = load_config_holder_contract(deps.storage)?;

    let config: Config = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Raw {
        contract_addr: config_holder_contract.to_string(),
        key: Binary::from(b"config"),
    }))?;

    Ok(config.nasset_token_rewards_contract)
}
