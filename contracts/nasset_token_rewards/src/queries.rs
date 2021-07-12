use crate::{
    math::decimal_summation_in_256,
    state::{Holder, HOLDERS},
    utils::calculate_decimal_rewards,
};
use cosmwasm_std::{Addr, Deps, Order, StdError, StdResult, Storage, Uint128};
use cw_storage_plus::Bound;
use yield_optimizer::nasset_token_rewards::{
    AccruedRewardsResponse, ConfigResponse, HolderResponse, HoldersResponse, StateResponse,
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

    let rewards = all_reward_with_decimals * Uint128(1);

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

//TODO: add `OrderBy` to query params
pub fn query_holders(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<HoldersResponse> {
    let start_after = if let Some(start_after) = start_after {
        Some(deps.api.addr_validate(&start_after)?)
    } else {
        None
    };

    let holders = load_holders(deps.storage, start_after, limit)?;
    Ok(HoldersResponse { holders })
}

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;
pub fn load_holders(
    storage: &dyn Storage,
    start_after: Option<Addr>,
    limit: Option<u32>,
) -> StdResult<Vec<HolderResponse>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = calc_range_start(start_after).map(Bound::exclusive);

    let holders: Result<Vec<_>, StdError> = HOLDERS
        .range(storage, start, None, Order::Ascending)
        .map(holder_to_response)
        .take(limit)
        .collect();

    Ok(holders?)
}

//idea copypasted from: https://github.com/CosmWasm/cosmwasm-plus/blob/3678c0c965431d4a8ebded636a02bb1a1f64b87c/packages/cw0/src/pagination.rs#L14
//TODO: there was a bug in that function, I fix it: https://github.com/CosmWasm/cosmwasm-plus/pull/316
//so we can use that function from library, when they merge
pub fn calc_range_start(start_after: Option<Addr>) -> Option<Vec<u8>> {
    start_after.map(|addr| {
        let mut v: Vec<u8> = addr.as_ref().into();
        v.push(0);
        v
    })
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
