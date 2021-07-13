use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use crate::error::ContractError;
use crate::state::{load_config, save_config, set_nasset_token_rewards_contract};
use crate::ContractResult;
use yield_optimizer::nasset_token_config_holder::{
    AnyoneMsg, Config, ConfigResponse, ExecuteMsg, GovernanceMsg, InstantiateMsg, QueryMsg,
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
    _env: Env,
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
        },

        ExecuteMsg::Governance { governance_msg } => {
            let config = load_config(deps.storage)?;
            if info.sender != config.governance_contract {
                return Err(ContractError::Unauthorized {});
            }

            match governance_msg {
                GovernanceMsg::UpdateConfig {
                    nasset_token_rewards_contract_addr: rewards_contract_addr,
                    governance_contract_addr,
                } => update_config(
                    deps,
                    config,
                    rewards_contract_addr,
                    governance_contract_addr,
                ),
            }
        }
    }
}

fn update_config(
    deps: DepsMut,
    mut current_config: Config,
    rewards_contract_addr: Option<String>,
    governance_contract_addr: Option<String>,
) -> ContractResult<Response> {
    if let Some(ref rewards_contract_addr) = rewards_contract_addr {
        current_config.nasset_token_rewards_contract =
            deps.api.addr_validate(rewards_contract_addr)?;
    }

    if let Some(ref governance_addr) = governance_contract_addr {
        current_config.governance_contract = deps.api.addr_validate(governance_addr)?;
    }

    save_config(deps.storage, &current_config)?;
    Ok(Response::default())
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config => to_binary(&query_config(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = load_config(deps.storage)?;
    Ok(ConfigResponse {
        nasset_token_rewards_addr: config.nasset_token_rewards_contract.to_string(),
        governance_contract_addr: config.governance_contract.to_string(),
    })
}
