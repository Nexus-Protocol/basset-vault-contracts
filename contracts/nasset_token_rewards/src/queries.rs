use crate::{
    math::decimal_summation_in_256,
    state::{Holder, PREFIX_HOLDERS},
    utils::calculate_decimal_rewards,
};
use cosmwasm_std::{Addr, Deps, StdResult, Storage, Uint128};
use cosmwasm_storage::ReadonlyBucket;
use basset_vault::{
    common::OrderBy,
    nasset_token_rewards::{
        AccruedRewardsResponse, ConfigResponse, HolderResponse, HoldersResponse, StateResponse,
    },
};

use crate::state::{load_config, load_holder, load_state, Config};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = load_config(deps.storage)?;
    Ok(ConfigResponse {
        psi_token_addr: config.psi_token.to_string(),
        nasset_token_addr: config.nasset_token.to_string(),
        governance_contract_addr: config.governance_contract.to_string(),
    })
}

pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = load_state(deps.storage)?;

    Ok(StateResponse {
        global_index: state.global_index,
        total_balance: state.total_balance,
        prev_reward_balance: state.prev_reward_balance,
    })
}

pub fn query_accrued_rewards(deps: Deps, address: String) -> StdResult<AccruedRewardsResponse> {
    let global_index = load_state(deps.storage)?.global_index;
    let holder_addr = deps.api.addr_validate(&address)?;

    let holder: Holder = load_holder(deps.storage, &holder_addr)?;
    let reward_with_decimals =
        calculate_decimal_rewards(global_index, holder.index, holder.balance)?;
    let all_reward_with_decimals =
        decimal_summation_in_256(reward_with_decimals, holder.pending_rewards);

    let rewards = all_reward_with_decimals * Uint128::new(1);

    Ok(AccruedRewardsResponse { rewards })
}

pub fn query_holder(deps: Deps, address: String) -> StdResult<HolderResponse> {
    let holder_addr = deps.api.addr_validate(&address)?;
    let holder: Holder = load_holder(deps.storage, &holder_addr)?;
    Ok(HolderResponse {
        address,
        balance: holder.balance,
        index: holder.index,
        pending_rewards: holder.pending_rewards,
    })
}

pub fn query_holders(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> StdResult<HoldersResponse> {
    let start_after = if let Some(start_after) = start_after {
        Some(deps.api.addr_validate(&start_after)?)
    } else {
        None
    };

    let holders = load_holders(deps.storage, start_after, limit, order_by)?;
    Ok(HoldersResponse { holders })
}

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;
pub fn load_holders(
    storage: &dyn Storage,
    start_after: Option<Addr>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> StdResult<Vec<HolderResponse>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let (start, end, order_by) = match order_by {
        Some(OrderBy::Asc) => (calc_range_start(start_after), None, OrderBy::Asc),
        _ => (None, calc_range_end(start_after), OrderBy::Desc),
    };

    let holders: ReadonlyBucket<Holder> = ReadonlyBucket::new(storage, PREFIX_HOLDERS);
    holders
        .range(start.as_deref(), end.as_deref(), order_by.into())
        .map(holder_to_response)
        .take(limit)
        .collect()
}

pub fn calc_range_start(start_after: Option<Addr>) -> Option<Vec<u8>> {
    start_after.map(|addr| {
        let mut v: Vec<u8> = addr.as_bytes().to_vec();
        v.push(0);
        v
    })
}

// this will set the first key after the provided key, by appending a 1 byte
fn calc_range_end(start_after: Option<Addr>) -> Option<Vec<u8>> {
    start_after.map(|addr| addr.as_bytes().to_vec())
}

pub fn holder_to_response(
    holder_with_address: StdResult<(Vec<u8>, Holder)>,
) -> StdResult<HolderResponse> {
    let (addr_bytes, holder) = holder_with_address?;
    let address = std::str::from_utf8(&addr_bytes)?.to_string();

    Ok(HolderResponse {
        address,
        balance: holder.balance,
        index: holder.index,
        pending_rewards: holder.pending_rewards,
    })
}
