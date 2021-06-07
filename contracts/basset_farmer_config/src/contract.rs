use std::collections::VecDeque;

use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Addr, Binary, CanonicalAddr, Deps, DepsMut, Env,
    MessageInfo, Reply, ReplyOn, Response, StdError, StdResult, SubMsg, WasmMsg,
};

use crate::{
    commands, queries,
    state::{save_state, PRICES_COUNT},
};
use crate::{error::ContractError, state::State};
use crate::{
    state::{Config, CONFIG},
    ContractResult,
};
use cw20::{Cw20ReceiveMsg, MinterResponse};
use protobuf::Message;
use yield_optimizer::basset_farmer_config::{
    ExecuteMsg, GovernanceMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    let config = Config {
        governance_contract_addr: deps.api.addr_validate(&msg.governance_contract_addr)?,
        borrow_ration_aim: msg.borrow_ration_aim,
        borrow_ration_upper_gap: msg.borrow_ration_upper_gap,
        borrow_ration_bottom_gap: msg.borrow_ration_bottom_gap,
        oracle_addr: deps.api.addr_validate(&msg.oracle_addr)?,
        basset_token_addr: deps.api.addr_validate(&msg.basset_token_addr)?,
        stable_denom: msg.stable_denom,
        price_timeframe_millis: msg.price_timeframe_millis,
    };

    CONFIG.save(deps.storage, &config)?;

    let state = State {
        prices: VecDeque::with_capacity(PRICES_COUNT as usize),
        price_last_update_time: 0,
        last_std_dev_from_average_price: Decimal256::zero(),
    };
    save_state(deps.storage, &state)?;

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
        ExecuteMsg::UpdatePrice {} => commands::update_price(deps, env, info),
        ExecuteMsg::GovernanceMsg { overseer_msg } => match overseer_msg {
            GovernanceMsg::UpdateConfig {
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
