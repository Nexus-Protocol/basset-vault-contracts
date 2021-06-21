use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Addr, Binary, CanonicalAddr, Coin, ContractInfo,
    CosmosMsg, Decimal, Deps, DepsMut, Empty, Env, MessageInfo, QueryRequest, Reply, ReplyOn,
    Response, StdError, StdResult, SubMsg, Uint128, WasmMsg, WasmQuery,
};
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::Cw20HookMsg as TerraswapCw20HookMsg;
use terraswap::pair::ExecuteMsg as TerraswapExecuteMsg;

use crate::utils;
use crate::{
    commands, queries,
    state::{load_config, store_staker_state, store_state, StakerState},
};
use crate::{
    error::ContractError,
    state::{load_staker_info, load_state, State},
};
use crate::{state::Config, ContractResult};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cw20::Cw20ReceiveMsg;
use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;
use yield_optimizer::basset_farmer::{CAssetStakerMsg, ExecuteMsg as BAssetExecuteMsg};
use yield_optimizer::{
    casset_staking::{AnyoneMsg, Cw20HookMsg, ExecuteMsg},
    get_tax_info,
    querier::{
        query_aterra_state, query_balance, query_borrower_info, query_token_balance,
        BorrowerInfoResponse,
    },
};

pub fn update_global_index(deps: DepsMut, env: Env, info: MessageInfo) -> ContractResult<Response> {
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
    let casset_addr = info.sender;
    // only bAsset contract can execute this message
    let config: Config = load_config(deps.storage)?;
    if casset_addr != config.casset_token {
        return Err(ContractError::Unauthorized {});
    }

    //we trust cw20 contract
    let staker_addr: Addr = Addr::unchecked(cw20_msg.sender);

    stake_casset(deps, env, config, staker_addr, cw20_msg.amount.into())
}

pub fn stake_casset(
    deps: DepsMut,
    env: Env,
    config: Config,
    staker: Addr,
    stake_amount: Uint256,
) -> ContractResult<Response> {
    let mut staker_state = load_staker_info(deps.storage, &staker)?;
    let mut state = load_state(deps.storage)?;

    utils::update_global_reward(deps.as_ref(), env, &config, &mut state)?;
    utils::update_staker_reward(&state, &mut staker_state);

    staker_state.staked_amount += stake_amount;

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

    let mut staker_state = load_staker_info(deps.storage, &staker)?;
    if staker_state.staked_amount < amount_to_unstake {
        return Err(StdError::generic_err("not enought casset to unstake").into());
    }

    let config: Config = load_config(deps.storage)?;
    let mut state = load_state(deps.storage)?;

    utils::update_global_reward(deps.as_ref(), env, &config, &mut state)?;
    utils::update_staker_reward(&state, &mut staker_state);

    let claim_amount = staker_state.pending_rewards * Uint256::one();
    let decimal_claim_amount = Decimal256::from_uint256(claim_amount);
    staker_state.pending_rewards = staker_state.pending_rewards - decimal_claim_amount;
    //TODO: write test on: Stake -> wait for reward -> Unstake -> Stake. Rewards amount for user
    // after second 'Stake' should be zero!
    state.last_reward_amount = state.last_reward_amount - decimal_claim_amount;
    staker_state.staked_amount = staker_state.staked_amount - amount_to_unstake;

    store_state(deps.storage, &state)?;
    store_staker_state(deps.storage, &staker, &staker_state)?;

    let mut messages: Vec<CosmosMsg> = vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.casset_token.to_string(),
        send: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: recipient.clone(),
            amount: amount_to_unstake.into(),
        })?,
    })];

    if !claim_amount.is_zero() {
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.basset_farmer_contract.to_string(),
            send: vec![],
            msg: to_binary(&BAssetExecuteMsg::CAssetStaker {
                casset_staker_msg: CAssetStakerMsg::SendRewards {
                    recipient,
                    amount: claim_amount,
                },
            })?,
        }))
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
    let mut staker_state = load_staker_info(deps.storage, &staker)?;

    utils::update_global_reward(deps.as_ref(), env, &config, &mut state)?;
    utils::update_staker_reward(&state, &mut staker_state);

    let claim_amount = staker_state.pending_rewards * Uint256::one();
    let decimal_claim_amount = Decimal256::from_uint256(claim_amount);
    staker_state.pending_rewards = staker_state.pending_rewards - decimal_claim_amount;
    state.last_reward_amount = state.last_reward_amount - decimal_claim_amount;

    store_state(deps.storage, &state)?;
    store_staker_state(deps.storage, &staker, &staker_state)?;

    let messages: Vec<CosmosMsg> = if !claim_amount.is_zero() {
        vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.basset_farmer_contract.to_string(),
            send: vec![],
            msg: to_binary(&BAssetExecuteMsg::CAssetStaker {
                casset_staker_msg: CAssetStakerMsg::SendRewards {
                    recipient,
                    amount: claim_amount,
                },
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
