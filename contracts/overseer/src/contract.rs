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
use yield_optimizer::{
    overseer::{MaintainerMsg, UserMsg},
    vault::ExecuteMsg as VaultHandleMsg,
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
        ExecuteMsg::MaintainerMsg { mantainer_msg } => {
            execute_maintainer_msg(deps, env, info, mantainer_msg)
        }
        ExecuteMsg::UserMsg { user_msg } => execute_user_msg(deps, env, info, user_msg),
    }
}

pub fn execute_maintainer_msg(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: MaintainerMsg,
) -> Result<Response, ContractError> {
    match msg {
        MaintainerMsg::Whitelist {
            name,
            collateral_token,
            strategy_contract,
        } => {
            //overseer whitelist response:
            //{
            //  "elems": [
            //    {
            //      "name": "Bonded Luna",
            //      "symbol": "BLUNA",
            //      "max_ltv": "0.5",
            //      "custody_contract": "terra1ptjp2vfjrwh0j0faj9r6katm640kgjxnwwq9kn",
            //      "collateral_token": "terra1kc87mu460fwkqte29rquh4hc20m54fxwtsx7gp"
            //    }
            //  ]
            // }
            let assets = tokens.validate(deps.api)?;
            commands::deposit_tokens(deps, info, assets)
        }

        MaintainerMsg::UpdateWhitelist {
            collateral_token,
            custody_contract,
            max_ltv,
        } => {
            let assets = tokens.validate(deps.api)?;
            commands::deposit_tokens(deps, info, assets)
        }
    }
}

pub fn execute_user_msg(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: UserMsg,
) -> Result<Response, ContractError> {
    match msg {
        UserMsg::Deposit { asset } => {
            let asset = asset.into_asset(deps.as_ref())?;
            commands::deposit_asset(deps, info, asset)
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Assets { depositor } => to_binary(&query::query_assets(deps, depositor)?),
        QueryMsg::AllAssets { start_after, limit } => {
            to_binary(&query::query_all_depositors(deps, start_after, limit)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}
