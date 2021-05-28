use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Addr, Api, Binary, CanonicalAddr, CosmosMsg, Deps,
    DepsMut, Env, MessageInfo, Order, Response, StdResult, WasmMsg,
};
use cw_storage_plus::Bound;

use crate::state::{
    depositor_to_human, read_deposits, read_whitelist_elem, store_deposits, State,
    DEPOSITED_TOKENS, STATE,
};
use crate::{error::ContractError, state::WhitelistElem};
use cw0::calc_range_start_human;
use cw20::Cw20ReceiveMsg;
use yield_optimizer::{
    custody::HandleMsg as CustodyHandleMsg,
    overseer::{AllDepositorsResponse, TokensResponse},
};
use yield_optimizer::{
    overseer::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    tokens::{Tokens, TokensHuman, TokensMath, TokensToHuman, TokensToRaw},
};

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        count: 22,
        owner: info.sender,
    };
    STATE.save(deps.storage, &state)?;

    Ok(Response::default())
}

// And declare a custom Error variant for the ones where you will want to make use of it
#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::DepositTokens { tokens: assets } => deposit_tokens(deps, info, assets),
    }
}

pub fn deposit_tokens(
    deps: DepsMut,
    info: MessageInfo,
    tokens_human: TokensHuman,
) -> Result<Response, ContractError> {
    let depositor_raw = deps.api.addr_canonicalize(&info.sender.to_string())?;
    let mut cur_deposits: Tokens = read_deposits(deps.storage, &depositor_raw)?;

    let tokens: Tokens = tokens_human.to_raw(deps.api)?;

    cur_deposits.add(tokens.clone());
    store_deposits(deps.storage, &depositor_raw, &cur_deposits)?;

    let mut messages: Vec<CosmosMsg> = vec![];
    for token in tokens {
        let whitelist_elem: WhitelistElem = read_whitelist_elem(deps.storage, &token.0)?;
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps
                .api
                .addr_humanize(&whitelist_elem.custody_contract)?
                .to_string(),
            send: vec![],
            msg: to_binary(&CustodyHandleMsg::LockTokens {
                depositor: info.sender.clone(),
                amount: token.1,
            })?,
        }));
    }

    // Logging stuff, so can be removed
    let tokens_logs: Vec<String> = tokens_human
        .iter()
        .map(|t| format!("{}{}", t.1, t.0.to_string()))
        .collect();

    Ok(Response {
        messages,
        attributes: vec![
            attr("action", "deposit_tokens"),
            attr("depositor", info.sender),
            attr("tokens", tokens_logs.join(",")),
        ],
        data: None,
        submessages: vec![],
    })
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Tokens { depositor } => to_binary(&query_tokens(deps, depositor)?),
        QueryMsg::AllTokens { start_after, limit } => {
            to_binary(&query_all_depositors(deps, start_after, limit)?)
        }
    }
}

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

pub fn query_tokens(deps: Deps, depositor: String) -> StdResult<TokensResponse> {
    let canonical_addr = deps.api.addr_canonicalize(&depositor)?;
    let tokens: Tokens = read_deposits(deps.storage, &canonical_addr)?;

    Ok(TokensResponse {
        //addr_canonicalize returns OK, so we can be sure that 'depositor' is a valid Address
        depositor: Addr::unchecked(depositor),
        tokens: tokens.to_human(deps.api)?,
    })
}

pub fn query_all_depositors(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<AllDepositorsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start =
        calc_range_start_human(deps.api, start_after.map(Addr::unchecked))?.map(Bound::exclusive);

    let depositors: Result<Vec<_>, _> = DEPOSITED_TOKENS
        .range(deps.storage, start, None, Order::Ascending)
        .map(|kv_res| depositor_to_human(deps.api, kv_res))
        .take(limit)
        .collect();

    Ok(AllDepositorsResponse {
        depositors: depositors?,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}
