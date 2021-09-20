use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, BlockInfo, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult,
};

use crate::error::ContractError;
use crate::state::{
    load_config, load_gov_update, remove_gov_update, save_config, save_gov_update,
    set_nasset_token_rewards_contract, GovernanceUpdateState,
};
use crate::ContractResult;
use basset_vault::nasset_token_config_holder::{
    AnyoneMsg, Config, ConfigResponse, ExecuteMsg, GovernanceMsg, InstantiateMsg, MigrateMsg,
    QueryMsg,
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    let config = Config {
        nasset_token_rewards_contract: Addr::unchecked(""),
        governance_contract: deps.api.addr_validate(&msg.governance_contract_addr)?,
    };

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
            AnyoneMsg::SetTokenRewardsContract {
                nasset_token_rewards_contract_addr,
            } => {
                let config = load_config(deps.storage)?;
                if config.nasset_token_rewards_contract.to_string().is_empty() {
                    let addr = deps
                        .api
                        .addr_validate(&nasset_token_rewards_contract_addr)?;

                    set_nasset_token_rewards_contract(deps.storage, addr)?;

                    Ok(Response::default())
                } else {
                    return Err(ContractError::Unauthorized {});
                }
            }

            AnyoneMsg::AcceptGovernance {} => accept_governance(deps, env, info),
        },

        ExecuteMsg::Governance { governance_msg } => {
            let config = load_config(deps.storage)?;
            if info.sender != config.governance_contract {
                return Err(ContractError::Unauthorized {});
            }

            match governance_msg {
                GovernanceMsg::UpdateConfig {
                    nasset_token_rewards_contract_addr: rewards_contract_addr,
                } => update_config(deps, config, rewards_contract_addr),

                GovernanceMsg::UpdateGovernanceContract {
                    gov_addr,
                    seconds_to_wait_for_accept_gov_tx,
                } => update_governance_addr(deps, env, gov_addr, seconds_to_wait_for_accept_gov_tx),
            }
        }
    }
}

fn update_config(
    deps: DepsMut,
    mut current_config: Config,
    rewards_contract_addr: Option<String>,
) -> ContractResult<Response> {
    if let Some(ref rewards_contract_addr) = rewards_contract_addr {
        current_config.nasset_token_rewards_contract =
            deps.api.addr_validate(rewards_contract_addr)?;
    }

    save_config(deps.storage, &current_config)?;
    Ok(Response::default())
}

pub fn update_governance_addr(
    deps: DepsMut,
    env: Env,
    gov_addr: String,
    seconds_to_wait_for_accept_gov_tx: u64,
) -> ContractResult<Response> {
    let current_time = get_time(&env.block);
    let gov_update = GovernanceUpdateState {
        new_governance_contract_addr: deps.api.addr_validate(&gov_addr)?,
        wait_approve_until: current_time + seconds_to_wait_for_accept_gov_tx,
    };
    save_gov_update(deps.storage, &gov_update)?;
    Ok(Response::default())
}

pub fn accept_governance(deps: DepsMut, env: Env, info: MessageInfo) -> ContractResult<Response> {
    let gov_update = load_gov_update(deps.storage)?;
    let current_time = get_time(&env.block);

    if gov_update.wait_approve_until < current_time {
        return Err(StdError::generic_err("too late to accept governance owning").into());
    }

    if info.sender != gov_update.new_governance_contract_addr {
        return Err(ContractError::Unauthorized {});
    }

    let new_gov_add_str = gov_update.new_governance_contract_addr.to_string();

    let mut config = load_config(deps.storage)?;
    config.governance_contract = gov_update.new_governance_contract_addr;
    save_config(deps.storage, &config)?;
    remove_gov_update(deps.storage);

    Ok(Response::default().add_attributes(vec![
        ("action", "change_governance_contract"),
        ("new_address", &new_gov_add_str),
    ]))
}

fn get_time(block: &BlockInfo) -> u64 {
    block.time.seconds()
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = load_config(deps.storage)?;
    Ok(ConfigResponse {
        nasset_token_rewards_addr: config.nasset_token_rewards_contract.to_string(),
        governance_contract_addr: config.governance_contract.to_string(),
    })
}

#[entry_point]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
