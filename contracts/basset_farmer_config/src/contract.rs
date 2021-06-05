use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Addr, Binary, CanonicalAddr, Deps, DepsMut, Env,
    MessageInfo, Reply, ReplyOn, Response, StdError, StdResult, SubMsg, WasmMsg,
};

use crate::{commands, queries};
use crate::{error::ContractError, response::MsgInstantiateContractResponse};
use crate::{
    state::{Config, CONFIG},
    ContractResult,
};
use cw20::{Cw20ReceiveMsg, MinterResponse};
use protobuf::Message;
use yield_optimizer::basset_farmer_config::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, OverseerMsg, QueryMsg,
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    let config = Config {
        borrow_ration_aim: msg.borrow_ration_aim,
        borrow_ration_upper_gap: msg.borrow_ration_upper_gap,
        borrow_ration_bottom_gap: msg.borrow_ration_bottom_gap,
        oracle_addr: deps.api.addr_validate(&msg.oracle_addr)?,
        basset_token_addr: deps.api.addr_validate(&msg.basset_token_addr)?,
        stable_denom: msg.stable_denom,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::UpdatePirce {} => commands::update_price(),
        ExecuteMsg::OverseerMsg { overseer_msg } => match overseer_msg {
            OverseerMsg::UpdateConfig {
                borrow_ration_aim,
                borrow_ration_upper_gap,
                borrow_ration_bottom_gap,
                oracle_addr,
                basset_token_addr,
                stable_denom,
            } => commands::update_config(
                deps,
                env,
                info,
                borrow_ration_aim,
                borrow_ration_upper_gap,
                borrow_ration_bottom_gap,
                oracle_addr,
                basset_token_addr,
                stable_denom,
            ),
        },
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
        QueryMsg::State {} => to_binary(&queries::query_state(deps)?),
        QueryMsg::BorrowLimits {} => to_binary(&queries::borrow_limits()?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResult<Response> {
    Ok(Response::default())
}
