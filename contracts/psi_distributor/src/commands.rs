use cosmwasm_std::{
    attr, to_binary, CosmosMsg, DepsMut, Env, Response, StdError, StdResult, WasmMsg,
};

use crate::state::{load_config, store_config, RewardShare, RewardsDistribution};
use crate::{state::Config, ContractResult};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;
use yield_optimizer::querier::query_token_balance;

pub fn distribute_rewards(deps: DepsMut, env: Env) -> ContractResult<Response> {
    let config: Config = load_config(deps.storage)?;
    let psi_balance: Uint256 =
        query_token_balance(deps.as_ref(), &config.psi_token, &env.contract.address)?.into();

    if psi_balance.is_zero() {
        return Err(StdError::generic_err("psi balance is zero").into());
    }

    let mut messages = Vec::with_capacity(config.rewards_distribution.distribution().len());

    let mut attributes = Vec::with_capacity(config.rewards_distribution.distribution().len() * 2);
    attributes.push(attr("action", "rewards_distribution"));

    for reward_share in config.rewards_distribution.distribution().iter() {
        let reward = psi_balance * reward_share.share;
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.psi_token.to_string(),
            send: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: reward_share.recipient.to_string(),
                amount: reward.into(),
            })?,
        }));

        attributes.push(attr("recepient", reward_share.recipient.to_string()));
        attributes.push(attr("reward_amount", reward));
    }

    Ok(Response {
        messages,
        submessages: vec![],
        attributes,
        data: None,
    })
}

pub fn update_config(
    deps: DepsMut,
    mut current_config: Config,
    psi_token_addr: Option<String>,
    governance_contract_addr: Option<String>,
) -> ContractResult<Response> {
    if let Some(ref psi_token_addr) = psi_token_addr {
        current_config.psi_token = deps.api.addr_validate(psi_token_addr)?;
    }

    if let Some(ref governance_contract_addr) = governance_contract_addr {
        current_config.governance_contract = deps.api.addr_validate(governance_contract_addr)?;
    }

    store_config(deps.storage, &current_config)?;
    Ok(Response::default())
}

pub fn update_distribution(
    deps: DepsMut,
    mut current_config: Config,
    distribution: Vec<(String, u64)>,
) -> ContractResult<Response> {
    let rewards_share: StdResult<Vec<RewardShare>> = distribution
        .into_iter()
        .map(|(addr_str, percent)| {
            let rew = RewardShare {
                recipient: deps.api.addr_validate(&addr_str)?,
                share: Decimal256::percent(percent),
            };
            Ok(rew)
        })
        .collect();

    let distribution = RewardsDistribution::new(rewards_share?)?;
    current_config.rewards_distribution = distribution;

    store_config(deps.storage, &current_config)?;
    Ok(Response::default())
}
