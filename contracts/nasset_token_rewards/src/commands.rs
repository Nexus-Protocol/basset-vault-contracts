use basset_vault::querier::query_token_balance;
use cosmwasm_std::{
    to_binary, Addr, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdError, Uint128, WasmMsg,
};

use crate::{
    math::decimal_summation_in_256,
    state::{load_config, load_holder, load_state, save_config, save_state, Config, Holder, State},
    ContractResult,
};
use crate::{
    state::save_holder,
    utils::{calculate_decimal_rewards, get_decimals},
};
use cw20::Cw20ExecuteMsg;

pub fn update_config(
    deps: DepsMut,
    mut current_config: Config,
    psi_token: Option<String>,
    nasset_token: Option<String>,
    governance_contract: Option<String>,
) -> ContractResult<Response> {
    if let Some(ref psi_token) = psi_token {
        current_config.psi_token = deps.api.addr_validate(psi_token)?;
    }

    if let Some(ref nasset_token) = nasset_token {
        current_config.nasset_token = deps.api.addr_validate(nasset_token)?;
    }

    if let Some(ref governance_contract) = governance_contract {
        current_config.governance_contract = deps.api.addr_validate(governance_contract)?;
    }

    save_config(deps.storage, &current_config)?;
    Ok(Response::default())
}

pub fn update_global_index(deps: DepsMut, env: Env) -> ContractResult<Response> {
    let mut state: State = load_state(deps.storage)?;

    // Zero nasset balance check
    if state.total_balance.is_zero() {
        return Err(StdError::generic_err("nAsset balance is zero").into());
    }

    let config = load_config(deps.storage)?;

    let claimed_rewards = calculate_global_index(deps.as_ref(), env, &config, &mut state)?;
    if claimed_rewards.is_zero() {
        return Err(StdError::generic_err("No rewards have accrued yet").into());
    }

    save_state(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "update_global_index"),
        ("claimed_rewards", &claimed_rewards.to_string()),
    ]))
}

fn calculate_global_index(
    deps: Deps,
    env: Env,
    config: &Config,
    state: &mut State,
) -> ContractResult<Uint128> {
    let balance = query_token_balance(deps, &config.psi_token, &env.contract.address)?;

    let previous_balance = state.prev_reward_balance;

    // claimed_rewards = current_balance - prev_balance;
    let claimed_rewards = balance.checked_sub(previous_balance)?;

    if claimed_rewards.is_zero() || state.total_balance.is_zero() {
        return Ok(claimed_rewards);
    }

    state.prev_reward_balance = balance;

    // global_index += claimed_rewards / total_balance;
    state.global_index = decimal_summation_in_256(
        state.global_index,
        Decimal::from_ratio(claimed_rewards, state.total_balance),
    );

    Ok(claimed_rewards)
}

pub fn claim_rewards(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Option<String>,
) -> ContractResult<Response> {
    let holder_addr = &info.sender;
    match recipient {
        Some(recipient) => {
            let recipient_addr = deps.api.addr_validate(&recipient)?;
            claim_rewards_logic(deps, env, holder_addr, &recipient_addr)
        }
        None => claim_rewards_logic(deps, env, holder_addr, holder_addr),
    }
}

pub fn claim_rewards_for_someone(
    deps: DepsMut,
    env: Env,
    recipient: String,
) -> ContractResult<Response> {
    let addr = deps.api.addr_validate(&recipient)?;
    claim_rewards_logic(deps, env, &addr, &addr)
}

fn claim_rewards_logic(
    deps: DepsMut,
    env: Env,
    holder_addr: &Addr,
    recipient: &Addr,
) -> ContractResult<Response> {
    let mut holder: Holder = load_holder(deps.storage, holder_addr)?;
    let mut state: State = load_state(deps.storage)?;
    let config: Config = load_config(deps.storage)?;

    calculate_global_index(deps.as_ref(), env, &config, &mut state)?;

    let reward_with_decimals =
        calculate_decimal_rewards(state.global_index, holder.index, holder.balance)?;

    let all_reward_with_decimals: Decimal =
        decimal_summation_in_256(reward_with_decimals, holder.pending_rewards);
    let decimals: Decimal = get_decimals(all_reward_with_decimals)?;

    let rewards: Uint128 = all_reward_with_decimals * Uint128::new(1);

    if rewards.is_zero() {
        return Err(StdError::generic_err("No rewards have accrued yet").into());
    }

    let new_balance = state.prev_reward_balance.checked_sub(rewards)?;
    state.prev_reward_balance = new_balance;
    save_state(deps.storage, &state)?;

    holder.pending_rewards = decimals;
    holder.index = state.global_index;
    save_holder(deps.storage, holder_addr, &holder)?;

    Ok(Response::new()
        .add_message(WasmMsg::Execute {
            contract_addr: config.psi_token.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: recipient.to_string(),
                amount: rewards,
            })?,
        })
        .add_attributes(vec![
            ("action", "claim_reward"),
            ("holder_address", &holder_addr.to_string()),
            ("recipient_address", &recipient.to_string()),
            ("rewards", &rewards.to_string()),
        ]))
}

pub fn increase_balance(
    deps: DepsMut,
    env: Env,
    config: &Config,
    address: String,
    amount: Uint128,
) -> ContractResult<Response> {
    let address = deps.api.addr_validate(&address)?;

    let mut state: State = load_state(deps.storage)?;
    let mut holder: Holder = load_holder(deps.storage, &address)?;

    // get decimals
    let rewards = calculate_decimal_rewards(state.global_index, holder.index, holder.balance)?;

    holder.index = state.global_index;
    holder.pending_rewards = decimal_summation_in_256(rewards, holder.pending_rewards);
    holder.balance += amount;
    state.total_balance += amount;

    calculate_global_index(deps.as_ref(), env, &config, &mut state)?;
    save_holder(deps.storage, &address, &holder)?;
    save_state(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "increase_balance"),
        ("holder_address", &address.to_string()),
        ("amount", &amount.to_string()),
    ]))
}

pub fn decrease_balance(
    deps: DepsMut,
    env: Env,
    config: &Config,
    address: String,
    amount: Uint128,
) -> ContractResult<Response> {
    let address = deps.api.addr_validate(&address)?;

    let mut state: State = load_state(deps.storage)?;
    let mut holder: Holder = load_holder(deps.storage, &address)?;

    if holder.balance < amount {
        return Err(StdError::generic_err(format!(
            "Decrease amount cannot exceed user balance: {}",
            holder.balance
        ))
        .into());
    }

    calculate_global_index(deps.as_ref(), env, &config, &mut state)?;

    // get decimals
    let rewards = calculate_decimal_rewards(state.global_index, holder.index, holder.balance)?;

    holder.index = state.global_index;
    holder.pending_rewards = decimal_summation_in_256(rewards, holder.pending_rewards);
    holder.balance = holder.balance.checked_sub(amount)?;
    state.total_balance = state.total_balance.checked_sub(amount)?;

    save_holder(deps.storage, &address, &holder)?;
    save_state(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "decrease_balance"),
        ("holder_address", &address.to_string()),
        ("amount", &amount.to_string()),
    ]))
}
