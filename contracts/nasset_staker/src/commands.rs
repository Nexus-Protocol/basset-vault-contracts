use cosmwasm_std::{
    attr, from_binary, to_binary, Addr, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError,
    Uint128, WasmMsg,
};

use crate::utils;
use crate::{
    commands,
    state::{load_config, store_staker_state, store_state},
};
use crate::{
    error::ContractError,
    state::{load_staker_state, load_state},
};
use crate::{state::Config, ContractResult};
use cosmwasm_bignumber::Uint256;
use cw20::Cw20ReceiveMsg;
use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;
use yield_optimizer::{nasset_staker::Cw20HookMsg, querier::query_token_balance};

pub fn update_global_index(deps: DepsMut, env: Env) -> ContractResult<Response> {
    let config = load_config(deps.storage)?;
    let mut state = load_state(deps.storage)?;
    utils::update_global_reward(deps.as_ref(), env, &config, &mut state)?;

    store_state(deps.storage, &state)?;

    Ok(Response {
        messages: vec![],
        submessages: vec![],
        attributes: vec![attr("action", "update_global_index")],
        data: None,
    })
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> ContractResult<Response> {
    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Stake) => commands::receive_cw20_stake(deps, env, info, cw20_msg),
        Err(err) => Err(ContractError::Std(err)),
    }
}

pub fn receive_cw20_stake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> ContractResult<Response> {
    let nasset_addr = info.sender;
    // only nAsset contract can execute this message
    let config: Config = load_config(deps.storage)?;
    if nasset_addr != config.nasset_token {
        return Err(ContractError::Unauthorized {});
    }

    //we trust cw20 contract
    let staker_addr: Addr = Addr::unchecked(cw20_msg.sender);

    stake_nasset(deps, env, config, staker_addr, cw20_msg.amount.into())
}

pub fn stake_nasset(
    deps: DepsMut,
    env: Env,
    config: Config,
    staker: Addr,
    stake_amount: Uint256,
) -> ContractResult<Response> {
    let mut staker_state = load_staker_state(deps.storage, &staker)?;
    let mut state = load_state(deps.storage)?;

    utils::update_global_reward(deps.as_ref(), env, &config, &mut state)?;
    utils::update_staker_reward(&state, &mut staker_state);

    utils::increase_staked_amount(&mut state, &mut staker_state, stake_amount);

    store_state(deps.storage, &state)?;
    store_staker_state(deps.storage, &staker, &staker_state)?;

    Ok(Response {
        messages: vec![],
        submessages: vec![],
        attributes: vec![attr("action", "stake"), attr("nasset_amount", stake_amount)],
        data: None,
    })
}

pub fn unstake_nasset(
    deps: DepsMut,
    env: Env,
    staker: Addr,
    amount_to_unstake: Uint256,
    to: Option<String>,
) -> ContractResult<Response> {
    let recipient = if let Some(to) = to {
        deps.api.addr_validate(&to)?.to_string()
    } else {
        staker.to_string()
    };

    let mut staker_state = load_staker_state(deps.storage, &staker)?;
    if staker_state.staked_amount < amount_to_unstake {
        return Err(StdError::generic_err("not enought nasset to unstake").into());
    }
    let config: Config = load_config(deps.storage)?;
    let mut state = load_state(deps.storage)?;

    utils::update_global_reward(deps.as_ref(), env, &config, &mut state)?;
    utils::update_staker_reward(&state, &mut staker_state);

    let claim_amount = utils::issue_reward(&mut state, &mut staker_state);
    utils::decrease_staked_amount(&mut state, &mut staker_state, amount_to_unstake);

    store_state(deps.storage, &state)?;
    store_staker_state(deps.storage, &staker, &staker_state)?;

    let mut messages = vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.nasset_token.to_string(),
        send: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: recipient.clone(),
            amount: amount_to_unstake.into(),
        })?,
    })];

    if !claim_amount.is_zero() {
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.psi_token.to_string(),
            send: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: recipient.clone(),
                amount: claim_amount.into(),
            })?,
        }));
    }

    Ok(Response {
        messages,
        submessages: vec![],
        attributes: vec![
            attr("action", "unstake"),
            attr("unstake_amount", amount_to_unstake),
            attr("claimed_rewards", claim_amount),
        ],
        data: None,
    })
}

pub fn claim_rewards(
    deps: DepsMut,
    env: Env,
    staker: Addr,
    recipient: Option<String>,
) -> ContractResult<Response> {
    let recipient = if let Some(recipient) = recipient {
        deps.api.addr_validate(&recipient)?.to_string()
    } else {
        staker.to_string()
    };

    let config: Config = load_config(deps.storage)?;
    let mut state = load_state(deps.storage)?;
    let mut staker_state = load_staker_state(deps.storage, &staker)?;

    utils::update_global_reward(deps.as_ref(), env, &config, &mut state)?;
    utils::update_staker_reward(&state, &mut staker_state);

    let claim_amount = utils::issue_reward(&mut state, &mut staker_state);

    store_state(deps.storage, &state)?;
    store_staker_state(deps.storage, &staker, &staker_state)?;

    let messages: Vec<CosmosMsg> = if !claim_amount.is_zero() {
        vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.psi_token.to_string(),
            send: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: recipient.clone(),
                amount: claim_amount.into(),
            })?,
        })]
    } else {
        vec![]
    };

    Ok(Response {
        messages,
        submessages: vec![],
        attributes: vec![
            attr("action", "claim_rewards"),
            attr("claimed_amount", claim_amount),
        ],
        data: None,
    })
}

pub fn claim_remainder(deps: DepsMut, env: Env) -> ContractResult<Response> {
    let config: Config = load_config(deps.storage)?;
    let state = load_state(deps.storage)?;

    if state.total_staked_amount.is_zero() {
        let psi_balance: Uint128 =
            query_token_balance(deps.as_ref(), &config.psi_token, &env.contract.address)?;

        if psi_balance.is_zero() {
            Err(StdError::generic_err("nothing to claim").into())
        } else {
            Ok(Response {
                messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: config.psi_token.to_string(),
                    send: vec![],
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: config.governance_addr.to_string(),
                        amount: psi_balance,
                    })?,
                })],
                submessages: vec![],
                attributes: vec![
                    attr("action", "claim_remainder"),
                    attr("amout", psi_balance),
                ],
                data: None,
            })
        }
    } else {
        Err(StdError::generic_err(format!(
            "wait until there will be 0 staked amount, currently: {}",
            state.total_staked_amount
        ))
        .into())
    }
}
