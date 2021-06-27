use cosmwasm_std::{attr, to_binary, CosmosMsg, DepsMut, Env, Response, WasmMsg};

use crate::state::load_config;
use crate::{state::Config, ContractResult};
use cosmwasm_bignumber::Uint256;
use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;
use yield_optimizer::querier::query_token_balance;

pub fn distribute_rewards(deps: DepsMut, env: Env) -> ContractResult<Response> {
    let config: Config = load_config(deps.storage)?;
    let nasset_balance: Uint256 = query_token_balance(
        deps.as_ref(),
        &config.nasset_token_addr,
        &env.contract.address,
    )?
    .into();

    let mut attributes =
        Vec::with_capacity(config.rewards_distribution.distribution().len() * 2 + 1);
    attributes.push(attr("action", "rewards_distribution"));

    if nasset_balance.is_zero() {
        attributes.push(attr("rewards_amount", "zero"));
        return Ok(Response {
            messages: vec![],
            submessages: vec![],
            attributes,
            data: None,
        });
    }

    let mut messages = Vec::with_capacity(config.rewards_distribution.distribution().len());

    for reward_share in config.rewards_distribution.distribution().iter() {
        let reward = nasset_balance * reward_share.share;
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.nasset_token_addr.to_string(),
            send: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: reward_share.recepient.to_string(),
                amount: reward.into(),
            })?,
        }));

        attributes.push(attr("recepient", reward_share.recepient.to_string()));
        attributes.push(attr("reward_amount", reward));
    }

    Ok(Response {
        messages,
        submessages: vec![],
        attributes,
        data: None,
    })
}
