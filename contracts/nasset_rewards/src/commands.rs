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
    oracle_addr: Option<String>,
    basset_token_addr: Option<String>,
    stable_denom: Option<String>,
    borrow_ltv_max: Option<Decimal256>,
    borrow_ltv_min: Option<Decimal256>,
    borrow_ltv_aim: Option<Decimal256>,
    basset_max_ltv: Option<Decimal256>,
    buffer_part: Option<Decimal256>,
) -> ContractResult<Response> {
    if let Some(ref oracle_addr) = oracle_addr {
        current_config.oracle_addr = deps.api.addr_validate(oracle_addr)?;
    }

    if let Some(ref basset_token_addr) = basset_token_addr {
        current_config.basset_token_addr = deps.api.addr_validate(basset_token_addr)?;
    }

    if let Some(stable_denom) = stable_denom {
        current_config.stable_denom = stable_denom;
    }

    if let Some(borrow_ltv_max) = borrow_ltv_max {
        current_config.borrow_ltv_max = borrow_ltv_max;
    }

    if let Some(borrow_ltv_min) = borrow_ltv_min {
        current_config.borrow_ltv_min = borrow_ltv_min;
    }

    if let Some(borrow_ltv_aim) = borrow_ltv_aim {
        current_config.borrow_ltv_aim = borrow_ltv_aim;
    }

    if let Some(basset_max_ltv) = basset_max_ltv {
        current_config.basset_max_ltv = basset_max_ltv;
    }

    if let Some(buffer_part) = buffer_part {
        current_config.buffer_part = buffer_part;
    }

    save_config(deps.storage, &current_config)?;
    Ok(Response::default())
}
