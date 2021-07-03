use cosmwasm_std::{DepsMut, Response};

use crate::{
    state::{save_config, Config},
    ContractResult,
};
use cosmwasm_bignumber::Decimal256;

/// Executor: governance
pub fn update_config(
    deps: DepsMut,
    mut current_config: Config,
    governance_addr: Option<String>,
    oracle_addr: Option<String>,
    basset_token_addr: Option<String>,
    stable_denom: Option<String>,
    borrow_ltv_max: Option<Decimal256>,
    borrow_ltv_min: Option<Decimal256>,
    borrow_ltv_aim: Option<Decimal256>,
    basset_max_ltv: Option<Decimal256>,
    buffer_part: Option<Decimal256>,
) -> ContractResult<Response> {
    if let Some(ref governance_addr) = governance_addr {
        current_config.governance_contract = deps.api.addr_validate(governance_addr)?;
    }

    if let Some(ref oracle_addr) = oracle_addr {
        current_config.oracle_contract = deps.api.addr_validate(oracle_addr)?;
    }

    if let Some(ref basset_token_addr) = basset_token_addr {
        current_config.basset_token = deps.api.addr_validate(basset_token_addr)?;
    }

    if let Some(stable_denom) = stable_denom {
        current_config.stable_denom = stable_denom;
    }

    current_config.validate_and_set_borrow_ltvs(
        borrow_ltv_max.unwrap_or(current_config.get_borrow_ltv_max()),
        borrow_ltv_min.unwrap_or(current_config.get_borrow_ltv_min()),
        borrow_ltv_aim.unwrap_or(current_config.get_borrow_ltv_aim()),
    )?;

    if let Some(basset_max_ltv) = basset_max_ltv {
        current_config.set_basset_max_ltv(basset_max_ltv)?;
    }

    if let Some(buffer_part) = buffer_part {
        current_config.set_buffer_part(buffer_part)?;
    }

    save_config(deps.storage, &current_config)?;
    Ok(Response::default())
}
