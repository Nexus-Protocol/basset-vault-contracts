use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use crate::{
    commands,
    error::ContractError,
    queries,
    state::{load_config, store_config, Config, RewardShare, RewardsDistribution},
    ContractResult,
};
use basset_vault::psi_distributor::{
    AnyoneMsg, ExecuteMsg, GovernanceMsg, InstantiateMsg, QueryMsg,
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    let nasset_token_rewards_addr = deps
        .api
        .addr_validate(&msg.nasset_token_rewards_contract_addr)?;
    let governance_contract = deps.api.addr_validate(&msg.governance_contract_addr)?;

    let rewards_distribution = vec![
        RewardShare {
            recipient: nasset_token_rewards_addr,
            share: Decimal256::percent(msg.nasset_token_rewards_share),
        },
        RewardShare {
            recipient: governance_contract.clone(),
            share: Decimal256::percent(msg.governance_contract_share),
        },
    ];

    let config = Config {
        psi_token: deps.api.addr_validate(&msg.psi_token_addr)?,
        governance_contract,
        rewards_distribution: RewardsDistribution::new(rewards_distribution)?,
    };
    store_config(deps.storage, &config)?;

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
            AnyoneMsg::DistributeRewards => commands::distribute_rewards(deps, env),
        },

        ExecuteMsg::GovernanceMsg { governance_msg } => {
            let config = load_config(deps.storage)?;
            if info.sender != config.governance_contract {
                return Err(ContractError::Unauthorized {});
            }

            match governance_msg {
                GovernanceMsg::UpdateConfig {
                    psi_token_contract_addr,
                    governance_contract_addr,
                } => commands::update_config(
                    deps,
                    config,
                    psi_token_contract_addr,
                    governance_contract_addr,
                ),

                GovernanceMsg::UpdateRewardsDistribution { distribution } => {
                    commands::update_distribution(deps, config, distribution)
                }
            }
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config => to_binary(&queries::query_config(deps)?),
    }
}
