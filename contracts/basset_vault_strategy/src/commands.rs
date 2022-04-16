use cosmwasm_std::{BlockInfo, DepsMut, Env, MessageInfo, Response, StdError};

use crate::{
    error::ContractError,
    state::{
        load_config, load_gov_update, remove_gov_update, save_config, save_gov_update, Config,
        GovernanceUpdateState,
    },
    ContractResult,
};
use cosmwasm_bignumber::Decimal256;

/// Executor: governance
#[allow(clippy::too_many_arguments)]
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
    price_timeframe: Option<u64>,
    anchor_market_addr: Option<String>,
    anchor_interest_model_addr: Option<String>,
    anchor_overseer_addr: Option<String>,
    anc_ust_swap_addr: Option<String>,
    anchor_token_addr: Option<String>,
    staking_apr: Option<Decimal256>,
) -> ContractResult<Response> {
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
        borrow_ltv_max.unwrap_or_else(|| current_config.get_borrow_ltv_max()),
        borrow_ltv_min.unwrap_or_else(|| current_config.get_borrow_ltv_min()),
        borrow_ltv_aim.unwrap_or_else(|| current_config.get_borrow_ltv_aim()),
    )?;

    if let Some(basset_max_ltv) = basset_max_ltv {
        current_config.set_basset_max_ltv(basset_max_ltv)?;
    }

    if let Some(buffer_part) = buffer_part {
        current_config.set_buffer_part(buffer_part)?;
    }

    if let Some(price_timeframe) = price_timeframe {
        current_config.price_timeframe = price_timeframe;
    }

    if let Some(ref anchor_market_addr) = anchor_market_addr {
        current_config.anchor_market_contract = deps.api.addr_validate(anchor_market_addr)?;
    }

    if let Some(ref anchor_interest_model_addr) = anchor_interest_model_addr {
        current_config.anchor_interest_model_contract = deps.api.addr_validate(anchor_interest_model_addr)?;
    }

    if let Some(ref anchor_overseer_addr) = anchor_overseer_addr {
        current_config.anchor_overseer_contract = deps.api.addr_validate(anchor_overseer_addr)?;
    }

    if let Some(ref anc_ust_swap_addr) = anc_ust_swap_addr {
        current_config.anc_ust_swap_contract = deps.api.addr_validate(anc_ust_swap_addr)?;
    }

    if let Some(ref anchor_token_addr) = anchor_token_addr {
        current_config.anchor_token = deps.api.addr_validate(anchor_token_addr)?;
    }

    if let Some(staking_apr) = staking_apr {
        current_config.staking_apr = staking_apr;
    }

    save_config(deps.storage, &current_config)?;
    Ok(Response::default())
}

pub fn update_governance_addr(
    deps: DepsMut,
    env: Env,
    gov_addr: String,
    seconds_to_wait_for_accept_gov_tx: u64,
) -> ContractResult<Response> {
    let current_time = get_time(&env.block);
    let gov_update = GovernanceUpdateState {
        new_governance_contract_addr: deps.api.addr_validate(&gov_addr)?,
        wait_approve_until: current_time + seconds_to_wait_for_accept_gov_tx,
    };
    save_gov_update(deps.storage, &gov_update)?;
    Ok(Response::default())
}

pub fn accept_governance(deps: DepsMut, env: Env, info: MessageInfo) -> ContractResult<Response> {
    let gov_update = load_gov_update(deps.storage)?;
    let current_time = get_time(&env.block);

    if gov_update.wait_approve_until < current_time {
        return Err(StdError::generic_err("too late to accept governance owning").into());
    }

    if info.sender != gov_update.new_governance_contract_addr {
        return Err(ContractError::Unauthorized);
    }

    let new_gov_add_str = gov_update.new_governance_contract_addr.to_string();

    let mut config = load_config(deps.storage)?;
    config.governance_contract = gov_update.new_governance_contract_addr;
    save_config(deps.storage, &config)?;
    remove_gov_update(deps.storage);

    Ok(Response::default().add_attributes(vec![
        ("action", "change_governance_contract"),
        ("new_address", &new_gov_add_str),
    ]))
}

fn get_time(block: &BlockInfo) -> u64 {
    block.time.seconds()
}
