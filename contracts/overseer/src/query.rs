use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Addr, Api, Binary, CanonicalAddr, CosmosMsg, Deps,
    DepsMut, Env, MessageInfo, Order, Response, StdResult, WasmMsg,
};
use cw_storage_plus::Bound;

use crate::state::{read_deposits, read_whitelist_elem, store_deposits, State, STATE};
use crate::{commands, query};
use crate::{error::ContractError, state::WhitelistElem};
use cw0::calc_range_start_human;
use cw20::Cw20ReceiveMsg;
use yield_optimizer::overseer::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use yield_optimizer::vault::ExecuteMsg as VaultHandleMsg;

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

//pub fn query_assets(deps: Deps, depositor: String) -> StdResult<TokensResponse> {
//    let canonical_addr = deps.api.addr_canonicalize(&depositor)?;
//    let tokens: TokensCanonical = read_deposits(deps.storage, &canonical_addr)?;

//    Ok(TokensResponse {
//        //addr_canonicalize returns OK, so we can be sure that 'depositor' is a valid Address
//        depositor: Addr::unchecked(depositor),
//        tokens: tokens.to_human(deps.api)?,
//    })
//}

//pub fn query_all_depositors(
//    deps: Deps,
//    start_after: Option<String>,
//    limit: Option<u32>,
//) -> StdResult<AllDepositorsResponse> {
//    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
//    let start =
//        calc_range_start_human(deps.api, start_after.map(Addr::unchecked))?.map(Bound::exclusive);

//    let depositors: Result<Vec<_>, _> = DEPOSITED_TOKENS
//        .range(deps.storage, start, None, Order::Ascending)
//        .map(|kv_res| depositor_to_human(deps.api, kv_res))
//        .take(limit)
//        .collect();

//    Ok(AllDepositorsResponse {
//        depositors: depositors?,
//    })
//}
