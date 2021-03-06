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
use basset_vault::basset_vault_strategy::{
    AnyoneMsg, ExecuteMsg, GovernanceMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    let config = Config::new(
        deps.api.addr_validate(&msg.governance_contract_addr)?,
        deps.api.addr_validate(&msg.oracle_contract_addr)?,
        deps.api.addr_validate(&msg.basset_token_addr)?,
        msg.stable_denom,
        msg.borrow_ltv_max,
        msg.borrow_ltv_min,
        msg.borrow_ltv_aim,
        msg.basset_max_ltv,
        msg.buffer_part,
        msg.price_timeframe,
    )?;

    save_config(deps.storage, &config)?;

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
        ExecuteMsg::Anyone { anyone_msg } => match anyone_msg {
            AnyoneMsg::AcceptGovernance {} => commands::accept_governance(deps, env, info),
        },

        ExecuteMsg::Governance { governance_msg } => {
            let config = load_config(deps.storage)?;
            if info.sender != config.governance_contract {
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
                    price_timeframe,
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
                    price_timeframe,
                ),

                GovernanceMsg::UpdateGovernanceContract {
                    gov_addr,
                    seconds_to_wait_for_accept_gov_tx,
                } => commands::update_governance_addr(
                    deps,
                    env,
                    gov_addr,
                    seconds_to_wait_for_accept_gov_tx,
                ),
            }
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
        QueryMsg::BorrowerAction {
            borrowed_amount,
            locked_basset_amount,
        } => to_binary(&queries::borrower_action(
            deps,
            env,
            borrowed_amount,
            locked_basset_amount,
        )?),
    }
}

#[entry_point]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
