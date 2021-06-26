use cosmwasm_std::{
    attr, from_binary, to_binary, Addr, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError,
    WasmMsg,
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
use cosmwasm_bignumber::{Decimal256, Uint256};
use cw20::Cw20ReceiveMsg;
use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;
use yield_optimizer::nasset_staker::Cw20HookMsg;

pub fn update_global_index(deps: DepsMut, env: Env) -> ContractResult<Response> {
    let config = load_config(deps.storage)?;
    let mut state = load_state(deps.storage)?;
    utils::update_global_reward(deps.as_ref(), env, &config, &mut state, None)?;

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

    utils::update_global_reward(deps.as_ref(), env, &config, &mut state, Some(stake_amount))?;
    utils::update_staker_reward(&state, &mut staker_state);

    utils::increase_staked_amount(&mut state, &mut staker_state, stake_amount);

    store_state(deps.storage, &state)?;
    store_staker_state(deps.storage, &staker, &staker_state)?;

    Ok(Response {
        messages: vec![],
        submessages: vec![],
        attributes: vec![attr("action", "stake"), attr("casset_amount", stake_amount)],
        data: None,
    })
}

pub fn unstake_casset(
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
        return Err(StdError::generic_err("not enought casset to unstake").into());
    }

    let config: Config = load_config(deps.storage)?;
    let mut state = load_state(deps.storage)?;

    utils::update_global_reward(deps.as_ref(), env, &config, &mut state, None)?;
    utils::update_staker_reward(&state, &mut staker_state);

    //TODO: add test on decimal part dropping. What if pending_rewards = 1.99?
    //How much claim_amount would be? It should be 1!
    let claim_amount = staker_state.pending_rewards * Uint256::one();
    staker_state.pending_rewards =
        staker_state.pending_rewards - Decimal256::from_uint256(claim_amount);
    //TODO: write test on: Stake -> wait for reward -> Unstake -> Stake. Rewards amount for user
    // after second 'Stake' should be zero!
    state.last_reward_amount = state.last_reward_amount - claim_amount;

    utils::decrease_staked_amount(&mut state, &mut staker_state, amount_to_unstake);

    store_state(deps.storage, &state)?;
    store_staker_state(deps.storage, &staker, &staker_state)?;

    Ok(Response {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.nasset_token.to_string(),
            send: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: recipient.clone(),
                amount: (amount_to_unstake + claim_amount).into(),
            })?,
        })],
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

    utils::update_global_reward(deps.as_ref(), env, &config, &mut state, None)?;
    utils::update_staker_reward(&state, &mut staker_state);

    let claim_amount = staker_state.pending_rewards * Uint256::one();
    staker_state.pending_rewards =
        staker_state.pending_rewards - Decimal256::from_uint256(claim_amount);
    state.last_reward_amount = state.last_reward_amount - claim_amount;

    store_state(deps.storage, &state)?;
    store_staker_state(deps.storage, &staker, &staker_state)?;

    let messages: Vec<CosmosMsg> = if !claim_amount.is_zero() {
        vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.nasset_token.to_string(),
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
