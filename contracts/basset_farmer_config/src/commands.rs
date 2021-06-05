use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Addr, Binary, CanonicalAddr, CosmosMsg, Decimal,
    Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn, Response, StdError, StdResult, SubMsg,
    Uint128, WasmMsg,
};

use crate::error::ContractError;
use crate::{commands, queries, state::load_config};
use crate::{
    state::{Config, CONFIG},
    ContractResult,
};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};
use yield_optimizer::{
    basset_farmer_config::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    querier::{get_basset_in_custody, query_supply, query_token_balance},
};

pub fn update_price() -> ContractResult<Response> {
    todo!()
}

/// Executor: overseer
pub fn update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    borrow_ration_aim: Option<Decimal>,
    borrow_ration_upper_gap: Option<Decimal>,
    borrow_ration_bottom_gap: Option<Decimal>,
    oracle_addr: Option<String>,
    basset_token_addr: Option<String>,
    stable_denom: Option<String>,
) -> ContractResult<Response> {
    let config = load_config(deps.storage)?;
    if info.sender != config.governance_contract_addr {
        return Err(ContractError::Unauthorized {});
    }

    todo!()
}
