use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use crate::{
    commands,
    error::ContractError,
    queries,
    state::{load_config, save_config},
};
use crate::{state::Config, ContractResult};
use yield_optimizer::basset_farmer_config::{
    ExecuteMsg, GovernanceMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    let config = Config {
        governance_contract_addr: deps.api.addr_validate(&msg.governance_contract_addr)?,
        oracle_addr: deps.api.addr_validate(&msg.oracle_addr)?,
        basset_token_addr: deps.api.addr_validate(&msg.basset_token_addr)?,
        stable_denom: msg.stable_denom,
        borrow_ltv_max: msg.borrow_ltv_max,
        borrow_ltv_min: msg.borrow_ltv_min,
        borrow_ltv_aim: msg.borrow_ltv_aim,
        basset_max_ltv: msg.basset_max_ltv,
        buffer_part: msg.buffer_part,
    };

    save_config(deps.storage, &config)?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::GovernanceMsg { governance_msg } => {
            let config = load_config(deps.storage)?;
            if info.sender != config.governance_contract_addr {
                return Err(ContractError::Unauthorized {});
            }

            match governance_msg {
                GovernanceMsg::UpdateConfig {
                    oracle_addr,
                    basset_token_addr,
                    stable_denom,
                    borrow_ltv_max,
                    borrow_ltv_min,
                    borrow_ltv_aim,
                    basset_max_ltv,
                    buffer_part,
                } => commands::update_config(
                    deps,
                    config,
                    oracle_addr,
                    basset_token_addr,
                    stable_denom,
                    borrow_ltv_max,
                    borrow_ltv_min,
                    borrow_ltv_aim,
                    basset_max_ltv,
                    buffer_part,
                ),
            }
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config => to_binary(&queries::query_config(deps)?),
        QueryMsg::BorrowerAction {
            borrowed_amount,
            locked_basset_amount,
        } => to_binary(&queries::borrower_action(
            deps,
            borrowed_amount,
            locked_basset_amount,
        )?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResult<Response> {
    Ok(Response::default())
}
