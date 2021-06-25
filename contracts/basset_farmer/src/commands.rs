use cosmwasm_std::{
    attr, from_binary, to_binary, Addr, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo,
    ReplyOn, Response, StdError, SubMsg, Uint128, WasmMsg,
};

use crate::{
    commands,
    contract::{SUBMSG_ID_BORROWING, SUBMSG_ID_REDEEM_STABLE},
    state::{
        load_aim_buffer_size, load_config, load_repaying_loan_state,
        load_stable_balance_before_selling_anc, store_aim_buffer_size,
        store_last_rewards_claiming_height, store_repaying_loan_state,
        store_stable_balance_before_selling_anc, RepayingLoanState,
    },
    utils::{
        calc_after_borrow_action, get_repay_loan_action, is_anc_rewards_claimable,
        split_profit_to_handle_interest,
    },
};
use crate::{error::ContractError, state::load_last_rewards_claiming_height};
use crate::{state::Config, ContractResult};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cw20::Cw20ReceiveMsg;
use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;
use yield_optimizer::{
    basset_farmer::{AnyoneMsg, Cw20HookMsg, ExecuteMsg, YourselfMsg},
    basset_farmer_config::{query_borrower_action, BorrowerActionResponse},
    get_tax_info,
    querier::{
        get_basset_in_custody, query_aterra_state, query_balance, query_borrower_info,
        query_supply, query_token_balance, AnchorMarketCw20Msg, AnchorMarketMsg, AnchorOverseerMsg,
        BorrowerInfoResponse,
    },
    terraswap_pair::Cw20HookMsg as TerraswapCw20HookMsg,
};

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> ContractResult<Response> {
    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Deposit) => commands::receive_cw20_deposit(deps, env, info, cw20_msg),
        Ok(Cw20HookMsg::Withdraw) => commands::receive_cw20_withdraw(deps, env, info, cw20_msg),
        Err(err) => Err(ContractError::Std(err)),
    }
}

pub fn receive_cw20_deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> ContractResult<Response> {
    let basset_addr = info.sender;
    // only bAsset contract can execute this message
    let config: Config = load_config(deps.storage)?;
    if basset_addr != config.basset_token {
        return Err(ContractError::Unauthorized {});
    }

    //we trust cw20 contract
    let farmer_addr: Addr = Addr::unchecked(cw20_msg.sender);

    deposit_basset(deps, env, config, farmer_addr, cw20_msg.amount.into())
}

pub fn deposit_basset(
    deps: DepsMut,
    env: Env,
    config: Config,
    farmer: Addr,
    deposit_amount: Uint256,
) -> ContractResult<Response> {
    let casset_supply: Uint256 = query_supply(&deps.querier, &config.nasset_token.clone())?.into();

    let basset_in_custody = get_basset_in_custody(
        deps.as_ref(),
        &config.anchor_custody_basset_contract,
        &env.contract.address,
    )?;

    // basset balance in cw20 contract
    let basset_in_contract_address =
        query_token_balance(deps.as_ref(), &config.basset_token, &env.contract.address)?;

    // cAsset tokens to mint:
    // user_share = (deposited_basset / total_basset)
    // cAsset_to_mint = cAsset_supply * user_share / (1 - user_share)
    let basset_balance: Uint256 = basset_in_custody + basset_in_contract_address.into();
    if basset_balance == Uint256::zero() {
        //impossible because 'farmer' already sent some basset
        return Err(ContractError::Impossible(
            "basset balance is zero".to_string(),
        ));
    }
    let farmer_basset_share: Decimal256 =
        Decimal256::from_ratio(deposit_amount.0, basset_balance.0);

    let casset_to_mint = if farmer_basset_share == Decimal256::one() {
        deposit_amount
    } else {
        // 'casset_supply' can't be zero here, cause we already mint some for first farmer
        casset_supply * farmer_basset_share / (Decimal256::one() - farmer_basset_share)
    };

    //1. lock basset
    //2. mint casset
    //3. rebalance
    Ok(Response {
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.anchor_overseer_contract.to_string(),
                msg: to_binary(&AnchorOverseerMsg::LockCollateral {
                    collaterals: vec![(config.basset_token.to_string(), deposit_amount)],
                })?,
                send: vec![],
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.nasset_token.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Mint {
                    recipient: farmer.to_string(),
                    amount: casset_to_mint.into(),
                })?,
                send: vec![],
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::Anyone {
                    anyone_msg: AnyoneMsg::Rebalance,
                })?,
                send: vec![],
            }),
        ],
        submessages: vec![],
        attributes: vec![
            attr("action", "deposit_basset"),
            attr("farmer", farmer),
            attr("amount", deposit_amount),
        ],
        data: None,
    })
}

pub fn receive_cw20_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> ContractResult<Response> {
    let contract_addr = info.sender;
    // only cAsset contract can execute this message
    let config: Config = load_config(deps.storage)?;
    if contract_addr != config.nasset_token {
        return Err(ContractError::Unauthorized {});
    }

    //we trust cw20 contract
    let farmer_addr: Addr = Addr::unchecked(cw20_msg.sender);

    withdraw_basset(deps, env, config, farmer_addr, cw20_msg.amount.into())
}

pub fn withdraw_basset(
    deps: DepsMut,
    env: Env,
    config: Config,
    farmer: Addr,
    casset_to_withdraw_amount: Uint256,
) -> ContractResult<Response> {
    //basset_in_contract_address is always zero (except Deposit stage)
    let basset_in_custody = get_basset_in_custody(
        deps.as_ref(),
        &config.anchor_custody_basset_contract,
        &env.contract.address,
    )?;

    let casset_token_supply = query_supply(&deps.querier, &config.nasset_token)?;

    let share_to_withdraw: Decimal256 = Decimal256::from_ratio(
        casset_to_withdraw_amount.0,
        Uint256::from(casset_token_supply).0,
    );
    let basset_to_withdraw: Uint256 = basset_in_custody * share_to_withdraw;

    //1. rebalance in a way you don't have basset_to_withdraw
    //2. unlock basset from custody
    //3. send basset to farmer
    //4. burn casset
    let mut rebalance_response = rebalance(
        deps,
        env,
        &config,
        basset_in_custody,
        Some(basset_to_withdraw),
    )?;

    rebalance_response
        .messages
        .push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.anchor_overseer_contract.to_string(),
            msg: to_binary(&AnchorOverseerMsg::UnlockCollateral {
                collaterals: vec![(config.basset_token.to_string(), basset_to_withdraw)],
            })?,
            send: vec![],
        }));

    rebalance_response
        .messages
        .push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.basset_token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: farmer.to_string(),
                amount: basset_to_withdraw.into(),
            })?,
            send: vec![],
        }));

    rebalance_response
        .messages
        .push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.nasset_token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Burn {
                amount: casset_to_withdraw_amount.into(),
            })?,
            send: vec![],
        }));

    rebalance_response
        .attributes
        .push(attr("action", "withdraw"));
    rebalance_response
        .attributes
        .push(attr("casset_amount", casset_to_withdraw_amount));

    Ok(rebalance_response)
}

/// Executor: anyone
pub fn rebalance(
    deps: DepsMut,
    env: Env,
    config: &Config,
    basset_in_custody: Uint256,
    basset_to_withdraw: Option<Uint256>,
) -> ContractResult<Response> {
    let basset_in_custody = basset_in_custody - basset_to_withdraw.unwrap_or_default();

    let borrower_info: BorrowerInfoResponse = query_borrower_info(
        deps.as_ref(),
        &config.anchor_market_contract,
        &env.contract.address,
    )?;
    let borrowed_ust = borrower_info.loan_amount;

    let borrower_action = query_borrower_action(
        deps.as_ref(),
        &config.basset_farmer_config_contract,
        borrowed_ust,
        basset_in_custody,
    )?;

    match borrower_action {
        BorrowerActionResponse::Nothing {} => {
            return Ok(Response {
                messages: vec![],
                submessages: vec![],
                attributes: vec![attr("action", "rebalance_not_needed")],
                data: None,
            })
        }

        BorrowerActionResponse::Borrow {
            amount,
            advised_buffer_size,
        } => {
            store_aim_buffer_size(deps.storage, &advised_buffer_size)?;
            borrow_logic(config, amount, advised_buffer_size)
        }

        BorrowerActionResponse::Repay {
            amount,
            advised_buffer_size,
        } => {
            store_aim_buffer_size(deps.storage, &advised_buffer_size)?;
            let mut repaying_loan_state = load_repaying_loan_state(deps.as_ref().storage)?;
            repaying_loan_state.to_repay_amount = amount;
            repaying_loan_state.aim_buffer_size = advised_buffer_size;
            repay_logic(deps, env, config, repaying_loan_state)
        }
    }
}

fn borrow_logic(
    config: &Config,
    borrow_amount: Uint256,
    aim_buffer_size: Uint256,
) -> ContractResult<Response> {
    Ok(Response {
        messages: vec![],
        submessages: vec![SubMsg {
            msg: WasmMsg::Execute {
                contract_addr: config.anchor_market_contract.to_string(),
                msg: to_binary(&AnchorMarketMsg::BorrowStable {
                    borrow_amount,
                    to: None,
                })?,
                send: vec![],
            }
            .into(),
            gas_limit: None,
            id: SUBMSG_ID_BORROWING,
            reply_on: ReplyOn::Success,
        }],
        attributes: vec![
            attr("action", "borrow_stable"),
            attr("amount", borrow_amount),
            attr("aim_buffer_size", aim_buffer_size),
        ],
        data: None,
    })
}

pub(crate) fn borrow_logic_on_reply(deps: DepsMut, env: Env) -> ContractResult<Response> {
    let config = load_config(deps.storage)?;
    let tax_info = get_tax_info(deps.as_ref(), &config.stable_denom)?;
    let aim_buf_size = load_aim_buffer_size(deps.as_ref().storage)?;
    let stable_coin_balance = query_balance(
        &deps.querier,
        &env.contract.address,
        config.stable_denom.clone(),
    )?;
    let after_borrow_action =
        calc_after_borrow_action(stable_coin_balance.into(), aim_buf_size, &tax_info);
    after_borrow_action.to_response(&config)
}

pub(crate) fn repay_logic(
    deps: DepsMut,
    env: Env,
    config: &Config,
    mut repaying_loan_state: RepayingLoanState,
) -> ContractResult<Response> {
    let aterra_balance =
        query_token_balance(deps.as_ref(), &config.aterra_token, &env.contract.address)?;
    let aterra_exchange_rate: Decimal256 =
        query_aterra_state(deps.as_ref(), &config.anchor_market_contract)?.exchange_rate;
    let stable_coin_balance = query_balance(
        &deps.querier,
        &env.contract.address,
        config.stable_denom.clone(),
    )?;

    let tax_info = get_tax_info(deps.as_ref(), &config.stable_denom)?;
    let repay_action = get_repay_loan_action(
        stable_coin_balance.into(),
        aterra_balance.into(),
        aterra_exchange_rate,
        repaying_loan_state.to_repay_amount,
        repaying_loan_state.aim_buffer_size,
        &tax_info,
        repaying_loan_state.iteration_index == 0,
    );

    repaying_loan_state.repaying_amount = repay_action.repaying_loan_amount();
    store_repaying_loan_state(deps.storage, &repaying_loan_state)?;

    repay_action.to_response(&config)
}

const LOAN_REPAYMENT_MAX_RECURSION_DEEP: u8 = 6;

pub(crate) fn repay_logic_on_reply(deps: DepsMut, env: Env) -> ContractResult<Response> {
    let mut repaying_loan_state = load_repaying_loan_state(deps.storage)?;
    repaying_loan_state.iteration_index += 1;
    if repaying_loan_state.iteration_index >= LOAN_REPAYMENT_MAX_RECURSION_DEEP {
        return Ok(Response::default());
    }
    let config = load_config(deps.storage)?;
    repay_logic(deps, env, &config, repaying_loan_state)
}

/// Anyone can execute claim_anc_rewards function to claim
/// ANC rewards, swap ANC => UST token, swap
/// part of UST => PSI token and distribute
/// result PSI token to gov contract
pub fn claim_anc_rewards(deps: DepsMut, env: Env) -> ContractResult<Response> {
    let config: Config = load_config(deps.storage)?;
    let last_rewards_claiming_height = load_last_rewards_claiming_height(deps.as_ref().storage)?;
    let current_height = env.block.height;

    if !is_anc_rewards_claimable(
        current_height,
        last_rewards_claiming_height,
        config.claiming_rewards_delay,
    ) {
        return Err(StdError::generic_err("claiming too often").into());
    }

    store_last_rewards_claiming_height(deps.storage, &current_height)?;

    Ok(Response {
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.anchor_market_contract.to_string(),
                msg: to_binary(&AnchorMarketMsg::ClaimRewards { to: None })?,
                send: vec![],
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::Yourself {
                    yourself_msg: YourselfMsg::SwapAnc,
                })?,
                send: vec![],
            }),
        ],
        submessages: vec![],
        attributes: vec![attr("action", "claim_anc_rewards")],
        data: None,
    })
}

pub fn swap_anc(deps: DepsMut, env: Env) -> ContractResult<Response> {
    let config: Config = load_config(deps.storage)?;

    let stable_coin_balance = query_balance(
        &deps.querier,
        &env.contract.address,
        config.stable_denom.clone(),
    )?;
    store_stable_balance_before_selling_anc(deps.storage, &stable_coin_balance)?;

    let anc_amount =
        query_token_balance(deps.as_ref(), &config.anchor_token, &env.contract.address)?;

    if anc_amount.is_zero() {
        return Err(StdError::generic_err("ANC amount is zero").into());
    }

    Ok(Response {
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.anchor_token.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Send {
                    amount: anc_amount,
                    contract: config.anc_stable_swap_contract.to_string(),
                    msg: to_binary(&TerraswapCw20HookMsg::Swap {
                        belief_price: None,
                        max_spread: None,
                        to: None,
                    })?,
                })?,
                send: vec![],
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::Yourself {
                    yourself_msg: YourselfMsg::DisributeRewards {},
                })?,
                send: vec![],
            }),
        ],
        submessages: vec![],
        attributes: vec![attr("action", "swap_anc"), attr("anc_swapped", anc_amount)],
        data: None,
    })
}

pub fn distribute_rewards(deps: DepsMut, env: Env) -> ContractResult<Response> {
    let config: Config = load_config(deps.storage)?;
    let stable_coin_balance_before_sell_anc =
        load_stable_balance_before_selling_anc(deps.as_ref().storage)?;

    let stable_coin_balance = query_balance(
        &deps.querier,
        &env.contract.address,
        config.stable_denom.clone(),
    )?;
    let aterra_balance =
        query_token_balance(deps.as_ref(), &config.aterra_token, &env.contract.address)?;

    let aterra_state = query_aterra_state(deps.as_ref(), &config.anchor_market_contract)?;
    let borrower_info: BorrowerInfoResponse = query_borrower_info(
        deps.as_ref(),
        &config.anchor_market_contract,
        &env.contract.address,
    )?;
    let borrowed_amount = borrower_info.loan_amount;

    let action_with_profit = split_profit_to_handle_interest(
        borrowed_amount,
        aterra_balance.into(),
        aterra_state.exchange_rate,
        stable_coin_balance.into(),
        stable_coin_balance_before_sell_anc.into(),
        config.over_loan_balance_value,
    );

    let tax_info = get_tax_info(deps.as_ref(), &config.stable_denom)?;

    action_with_profit.to_response(&config, &tax_info)
}
