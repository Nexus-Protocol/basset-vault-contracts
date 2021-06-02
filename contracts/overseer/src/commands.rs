use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Addr, Api, Binary, CanonicalAddr, CosmosMsg, Deps,
    DepsMut, Env, MessageInfo, Order, Response, StdResult, WasmMsg,
};
use cw_storage_plus::Bound;

use crate::state::{
    read_deposits, read_whitelist_elem, store_deposits, Config, State, DEPOSITED_ASSETS, STATE,
};
use crate::{commands, queries};
use crate::{error::ContractError, state::WhitelistElem};
use cw0::calc_range_start_human;
use cw20::Cw20ReceiveMsg;
use yield_optimizer::{asset::AssetInfoRaw, basset_farmer::ExecuteMsg as VaultHandleMsg};
use yield_optimizer::{
    asset::{AssetInfo, AssetString},
    overseer::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
};

pub fn deposit_asset(
    deps: DepsMut,
    info: MessageInfo,
    to_deposit: AssetInfo,
) -> Result<Response, ContractError> {
    // let depositor_raw = deps.api.addr_canonicalize(&info.sender.to_string())?;
    // let mut cur_deposits: Vec<AssetInfoRaw> = read_deposits(deps.storage, &depositor_raw)?;

    // let to_deposit_raw: AssetInfoRaw = to_deposit.to_raw(deps.as_ref())?;

    // cur_deposits.add(to_deposit_raw.clone());
    // store_deposits(deps.storage, &depositor_raw, &cur_deposits)?;

    // let mut messages: Vec<CosmosMsg> = vec![];
    // for token in tokens_canonical {
    //     let whitelist_elem: WhitelistElem = read_whitelist_elem(deps.storage, &token.0)?;
    //     messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
    //         contract_addr: deps
    //             .api
    //             .addr_humanize(&whitelist_elem.vault_contract)?
    //             .to_string(),
    //         send: vec![],
    //         msg: to_binary(&VaultHandleMsg::DepositTokens {
    //             depositor: info.sender.clone(),
    //             amount: token.1,
    //         })?,
    //     }));
    // }

    // // Logging stuff, so can be removed
    // let tokens_logs: Vec<String> = tokens_addr
    //     .iter()
    //     .map(|t| format!("{}{}", t.1, t.0.to_string()))
    //     .collect();

    Ok(Response {
        messages: vec![],
        attributes: vec![
            attr("action", "deposit_tokens"),
            attr("depositor", info.sender),
            // attr("tokens", tokens_logs.join(",")),
        ],
        data: None,
        submessages: vec![],
    })
}

pub fn register_whitelist(
    deps: DepsMut,
    info: MessageInfo,
    tokens_human: AssetString, //or AssetAddr
) -> Result<Response, ContractError> {
    // let config: Config = read_config(&deps.storage)?;
    // if deps.api.canonical_address(&env.message.sender)? != config.maintainer {
    //     return Err(StdError::unauthorized());
    // }

    // let collateral_token_raw = deps.api.canonical_address(&collateral_token)?;
    // if read_whitelist_elem(&deps.storage, &collateral_token_raw).is_ok() {
    //     return Err(StdError::generic_err(
    //         "Token is already registered as collateral",
    //     ));
    // }

    // store_whitelist_elem(
    //     &mut deps.storage,
    //     &collateral_token_raw,
    //     &WhitelistElem {
    //         name: name.to_string(),
    //         symbol: symbol.to_string(),
    //         vault_contract: deps.api.canonical_address(&custody_contract)?,
    //         max_ltv,
    //     },
    // )?;

    Ok(Response {
        messages: vec![],
        attributes: vec![
            attr("action", "register_whitelist"),
            // attr("name", name),
            // attr("symbol", symbol),
            // attr("collateral_token", collateral_token),
            // attr("custody_contract", custody_contract),
            // attr("LTV", max_ltv),
        ],
        data: None,
        submessages: vec![],
    })
}
