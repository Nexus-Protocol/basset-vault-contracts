use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};

use crate::{
    commands,
    error::ContractError,
    queries,
    state::{load_config, store_config, Config, RewardShare, RewardsDistribution},
    ContractResult,
};
use yield_optimizer::psi_distributor::{
    AnyoneMsg, ExecuteMsg, GovernanceMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};

const NASSET_TOKEN_HOLDERS_REWARDS_SHARE: u64 = 70;
const GOVERNANCE_STAKER_REWARDS_SHARE: u64 = 30;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    let nasset_token_rewards_addr = deps.api.addr_validate(&msg.nasset_token_rewards_contract)?;
    let governance_addr = deps.api.addr_validate(&msg.governance_contract)?;

    let rewards_distribution = vec![
        RewardShare {
            recipient: nasset_token_rewards_addr,
            share: Decimal256::percent(NASSET_TOKEN_HOLDERS_REWARDS_SHARE),
        },
        RewardShare {
            recipient: governance_addr.clone(),
            share: Decimal256::percent(GOVERNANCE_STAKER_REWARDS_SHARE),
        },
    ];

    let config = Config {
        nasset_token_addr: deps.api.addr_validate(&msg.nasset_token_contract)?,
        governance_addr,
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
            if info.sender != config.governance_addr {
                return Err(ContractError::Unauthorized {});
            }

            match governance_msg {
                GovernanceMsg::UpdateConfig {
                    nasset_token_addr,
                    governance_addr,
                } => commands::update_config(deps, config, nasset_token_addr, governance_addr),

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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> ContractResult<Response> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResult<Response> {
    Ok(Response::default())
}
