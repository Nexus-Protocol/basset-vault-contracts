use crate::{
    commands,
    state::{
        load_aim_buffer_size, load_config, load_gov_update, load_last_rewards_claiming_height,
        load_repaying_loan_state, load_stable_balance_before_selling_anc, remove_gov_update,
        store_aim_buffer_size, store_config, store_gov_update, store_last_rewards_claiming_height,
        store_repaying_loan_state, store_stable_balance_before_selling_anc, Config,
        GovernanceUpdateState, RepayingLoanState,
    },
    tax_querier::get_tax_info,
    utils::{
        calc_after_borrow_action, get_repay_loan_action, is_anc_rewards_claimable,
        split_profit_to_handle_interest,
    },
    SubmsgIds,
};
use basset_vault::{
    anchor::basset_custody::get_basset_in_custody,
    anchor::market::{query_borrower_info, BorrowerInfoResponse},
    basset_vault::{AnyoneMsg, Cw20HookMsg, ExecuteMsg, YourselfMsg},
    basset_vault_strategy::{query_borrower_action, BorrowerActionResponse},
    querier::{
        query_aterra_state, query_balance, query_supply, query_token_balance, AnchorCustodyCw20Msg,
        AnchorCustodyMsg, AnchorMarketCw20Msg, AnchorMarketMsg, AnchorOverseerMsg,
    },
    terraswap::{Asset, AssetInfo},
    terraswap_pair::{Cw20HookMsg as TerraswapCw20HookMsg, ExecuteMsg as TerraswapExecuteMsg},
    BASSET_VAULT_LOAN_REPAYMENT_MAX_RECURSION_DEEP,
};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    from_binary, to_binary, Addr, BlockInfo, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

pub fn update_config(
    deps: DepsMut,
    mut current_config: Config,
    psi_distributor_addr: Option<String>,
    anchor_overseer_contract_addr: Option<String>,
    anchor_market_contract_addr: Option<String>,
    anchor_custody_basset_contract_addr: Option<String>,
    anc_stable_swap_contract_addr: Option<String>,
    psi_stable_swap_contract_addr: Option<String>,
    basset_vault_strategy_contract_addr: Option<String>,
    claiming_rewards_delay: Option<u64>,
    over_loan_balance_value: Option<Decimal256>,
) -> StdResult<Response> {
    if let Some(ref psi_distributor_addr) = psi_distributor_addr {
        current_config.psi_distributor = deps.api.addr_validate(psi_distributor_addr)?;
    }

    if let Some(ref anchor_overseer_contract_addr) = anchor_overseer_contract_addr {
        current_config.anchor_overseer_contract =
            deps.api.addr_validate(anchor_overseer_contract_addr)?;
    }

    if let Some(ref anchor_market_contract_addr) = anchor_market_contract_addr {
        current_config.anchor_market_contract =
            deps.api.addr_validate(anchor_market_contract_addr)?;
    }

    if let Some(ref anchor_custody_basset_contract_addr) = anchor_custody_basset_contract_addr {
        current_config.anchor_custody_basset_contract = deps
            .api
            .addr_validate(anchor_custody_basset_contract_addr)?;
    }

    if let Some(ref anc_stable_swap_contract_addr) = anc_stable_swap_contract_addr {
        current_config.anc_stable_swap_contract =
            deps.api.addr_validate(anc_stable_swap_contract_addr)?;
    }

    if let Some(ref psi_stable_swap_contract_addr) = psi_stable_swap_contract_addr {
        current_config.psi_stable_swap_contract =
            deps.api.addr_validate(psi_stable_swap_contract_addr)?;
    }

    if let Some(ref basset_vault_strategy_contract_addr) = basset_vault_strategy_contract_addr {
        current_config.basset_vault_strategy_contract = deps
            .api
            .addr_validate(basset_vault_strategy_contract_addr)?;
    }

    if let Some(claiming_rewards_delay) = claiming_rewards_delay {
        current_config.claiming_rewards_delay = claiming_rewards_delay;
    }

    if let Some(over_loan_balance_value) = over_loan_balance_value {
        current_config.over_loan_balance_value = over_loan_balance_value;
    }

    store_config(deps.storage, &current_config)?;
    Ok(Response::default())
}

pub fn update_governance_addr(
    deps: DepsMut,
    env: Env,
    gov_addr: String,
    seconds_to_wait_for_accept_gov_tx: u64,
) -> StdResult<Response> {
    let current_time = get_time(&env.block);
    let gov_update = GovernanceUpdateState {
        new_governance_contract_addr: deps.api.addr_validate(&gov_addr)?,
        wait_approve_until: current_time + seconds_to_wait_for_accept_gov_tx,
    };
    store_gov_update(deps.storage, &gov_update)?;
    Ok(Response::default())
}

pub fn accept_governance(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let gov_update = load_gov_update(deps.storage)?;
    let current_time = get_time(&env.block);

    if gov_update.wait_approve_until < current_time {
        return Err(StdError::generic_err(
            "too late to accept governance owning",
        ));
    }

    if info.sender != gov_update.new_governance_contract_addr {
        return Err(StdError::generic_err("unauthorized"));
    }

    let new_gov_add_str = gov_update.new_governance_contract_addr.to_string();

    let mut config = load_config(deps.storage)?;
    config.governance_contract = gov_update.new_governance_contract_addr;
    store_config(deps.storage, &config)?;
    remove_gov_update(deps.storage);

    Ok(Response::default().add_attributes(vec![
        ("action", "change_governance_contract"),
        ("new_address", &new_gov_add_str),
    ]))
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    match from_binary(&cw20_msg.msg)? {
        Cw20HookMsg::Deposit {} => commands::receive_cw20_deposit(deps, env, info, cw20_msg),
        Cw20HookMsg::Withdraw {} => commands::receive_cw20_withdraw(deps, env, info, cw20_msg),
    }
}

pub fn receive_cw20_deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    let basset_addr = info.sender;
    // only bAsset contract can execute this message
    let config: Config = load_config(deps.storage)?;
    if basset_addr != config.basset_token {
        return Err(StdError::generic_err("unauthorized"));
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
    deposited_basset: Uint256,
) -> StdResult<Response> {
    let nasset_supply: Uint256 = query_supply(&deps.querier, &config.nasset_token.clone())?.into();

    let basset_in_custody = get_basset_in_custody(
        deps.as_ref(),
        &config.anchor_custody_basset_contract,
        &env.contract.address,
    )?;

    // TODO: this can happen now
    if basset_in_custody.is_zero() && !nasset_supply.is_zero() {
        //read comments in 'withdraw_basset' function for a reason to return error here
        return Err(StdError::generic_err(
            "bAsset balance is zero, but nAsset supply is not! Freeze contract.",
        ));
    }

    let basset_in_contract_address = query_token_balance(
        deps.as_ref(),
        &config.basset_token,
        &env.contract.address
    )
    .into();

    let basset_balance: Uint256 = basset_in_custody + basset_in_contract_address;

    if basset_balance == Uint256::zero() {
        //impossible because 'farmer' already sent some basset
        return Err(StdError::generic_err(
            "basset balance is zero (impossible case)".to_string(),
        ));
    }

    let is_first_depositor = nasset_supply.is_zero();

    // nAsset tokens to mint:
    // user_share = (deposited_basset / total_basset)
    // nAsset_to_mint = nAsset_supply * user_share / (1 - user_share)
    let nasset_to_mint = if is_first_depositor {
        basset_in_contract_address
    } else {
        // 'nasset_supply' can't be zero here, cause we already mint some for first farmer
        let user_share = Decimal256::from_uint256(deposited_basset) / Decimal256::from_uint256(basset_balance);
        nasset_supply * user_share / (Decimal256::one() - user_share)
    };

    // 1. Mint nasset
    // 2. Rebalance
    Ok(Response::new()
        .add_messages(vec![
            WasmMsg::Execute {
                contract_addr: config.nasset_token.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Mint {
                    recipient: farmer.to_string(),
                    amount: nasset_to_mint.into(),
                })?,
                funds: vec![],
            },
            WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::Anyone {
                    anyone_msg: AnyoneMsg::Rebalance {},
                })?,
                funds: vec![],
            },
        ])
        .add_attributes(vec![
            ("action", "deposit_basset"),
            ("farmer", &farmer.to_string()),
            ("amount", &deposited_basset.to_string()),
        ]))
}

pub fn receive_cw20_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    let contract_addr = info.sender;
    // only nAsset contract can execute this message
    let config: Config = load_config(deps.storage)?;
    if contract_addr != config.nasset_token {
        return Err(StdError::generic_err("unauthorized"));
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
    nasset_to_withdraw_amount: Uint256,
) -> StdResult<Response> {
    //nasset_to_withdraw_amount is not zero here, cw20 contract check it

    let basset_in_custody = get_basset_in_custody(
        deps.as_ref(),
        &config.anchor_custody_basset_contract,
        &env.contract.address,
    )?;

    let basset_balance: Uint256 = query_token_balance(
        deps.as_ref(),
        &config.basset_token,
        &env.contract.address,
    ).into();
    
    if (basset_in_custody + basset_balance).is_zero() {
        //interesting case - user owns some nAsset, but bAsset balance is zero
        //what we can do here:
        //1. Burn his nAsset, cause they do not have value in that context
        //2. return error. In that case if someone will deposit bAsset those nAsset owners will
        //   own share of his tokens. But I prevent deposists in that case, so contract is kinds "frozen" -
        //   no withdraw and deposits available when bLuna balance is zero. Looks like the best
        //   solution.
        //3. Burn all nAsset supply (not possible with cw20 messages)
        //
        //Second choice is best one in my opinion.
        return Err(StdError::generic_err(
            "bAsset balance is zero, but nAsset supply is not! Freeze contract.",
        ));
    }

    let nasset_token_supply = query_supply(&deps.querier, &config.nasset_token)?;

    let basset_to_withdraw: Uint256 = basset_in_custody * nasset_to_withdraw_amount
        / Decimal256::from_uint256(Uint256::from(nasset_token_supply));

    let mut response = Response::new();

    //
    // If amount of basset on the balance is not enough
    // then withdraw it from collateral:
    //
    // 1. rebalance in a way you don't have basset_to_withdraw
    // 2. unlock basset from anchor_overseer
    // 3. withdraw basset from anchor_custody
    //
    if basset_balance < basset_to_withdraw {
        let needed_additional_amount = basset_to_withdraw - basset_balance;

        response = rebalance(
            deps,
            env,
            &config,
            basset_in_custody,
            Some(needed_additional_amount),
        )?;
    
        response
            .messages
            .push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.anchor_overseer_contract.to_string(),
                msg: to_binary(&AnchorOverseerMsg::UnlockCollateral {
                    collaterals: vec![(config.basset_token.to_string(), needed_additional_amount)],
                })?,
                funds: vec![],
            })));
    
        response
            .messages
            .push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.anchor_custody_basset_contract.to_string(),
                msg: to_binary(&AnchorCustodyMsg::WithdrawCollateral {
                    amount: Some(needed_additional_amount),
                })?,
                funds: vec![],
            })));
    }

    // 1. send basset to farmer
    // 2. burn nasset

    response
        .messages
        .push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.basset_token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: farmer.to_string(),
                amount: basset_to_withdraw.into(),
            })?,
            funds: vec![],
        })));

    response
        .messages
        .push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.nasset_token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Burn {
                amount: nasset_to_withdraw_amount.into(),
            })?,
            funds: vec![],
        })));

    Ok(response.add_attributes(vec![
        ("action", "withdraw"),
        ("nasset_amount", &nasset_to_withdraw_amount.to_string()),
    ]))
}

/// Executor: anyone
pub fn rebalance(
    deps: DepsMut,
    env: Env,
    config: &Config,
    basset_in_custody: Uint256,
    basset_to_withdraw: Option<Uint256>,
) -> StdResult<Response> {
    let basset_in_custody = basset_in_custody - basset_to_withdraw.unwrap_or_default();

    let borrower_info: BorrowerInfoResponse = query_borrower_info(
        deps.as_ref(),
        &config.anchor_market_contract,
        &env.contract.address,
    )?;
    let borrowed_ust = borrower_info.loan_amount;

    let borrower_action = query_borrower_action(
        deps.as_ref(),
        &config.basset_vault_strategy_contract,
        borrowed_ust,
        basset_in_custody,
    )?;

    match borrower_action {
        BorrowerActionResponse::Nothing {} => {
            //maybe it is better to return error here, but
            //we cant, cause it is used in 'withdraw'
            return Ok(Response::new().add_attribute("action", "rebalance_not_needed"));
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
            let repaying_loan_state = RepayingLoanState {
                to_repay_amount: amount,
                aim_buffer_size: advised_buffer_size,
                ..RepayingLoanState::default()
            };
            repay_logic(deps, env, config, repaying_loan_state)
        }

        BorrowerActionResponse::DepositAll {} => deposit_all_logic(deps, env, config),

        BorrowerActionResponse::WithdrawAll {} => withdraw_all_logic(deps, env, config),
    }
}

fn borrow_logic(
    config: &Config,
    borrow_amount: Uint256,
    aim_buffer_size: Uint256,
) -> StdResult<Response> {
    // If can't borrow from Anchor we can't do anything, so just return error, consequence:
    // 1. user will not be able to deposit
    // 2. Rebalance return error
    Ok(Response::new()
        .add_submessage(SubMsg::reply_on_success(
            WasmMsg::Execute {
                contract_addr: config.anchor_market_contract.to_string(),
                msg: to_binary(&AnchorMarketMsg::BorrowStable {
                    borrow_amount,
                    to: None,
                })?,
                funds: vec![],
            },
            SubmsgIds::Borrowing.id(),
        ))
        .add_attributes(vec![
            ("action", "borrow_stable"),
            ("amount", &borrow_amount.to_string()),
            ("aim_buffer_size", &aim_buffer_size.to_string()),
        ]))
}

pub(crate) fn borrow_logic_on_reply(deps: DepsMut, env: Env) -> StdResult<Response> {
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
) -> StdResult<Response> {
    let aterra_balance =
        query_token_balance(deps.as_ref(), &config.aterra_token, &env.contract.address);
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

pub(crate) fn repay_logic_on_reply(deps: DepsMut, env: Env) -> StdResult<Response> {
    let mut repaying_loan_state = load_repaying_loan_state(deps.storage)?;
    repaying_loan_state.iteration_index += 1;
    if repaying_loan_state.iteration_index >= BASSET_VAULT_LOAN_REPAYMENT_MAX_RECURSION_DEEP {
        if repaying_loan_state.repayed_something {
            return Ok(Response::default());
        } else {
            return Err(StdError::generic_err("fail to repay loan").into());
        }
    }
    let config = load_config(deps.storage)?;
    repay_logic(deps, env, &config, repaying_loan_state)
}

pub(crate) fn deposit_all_logic(
    deps: DepsMut,
    env: Env,
    config: &Config,
) -> StdResult<Response> {
    let basset_in_contract_address = query_token_balance(
        deps.as_ref(),
        &config.basset_token,
        &env.contract.address
    );

    // 1. send basset to anchor_custody contract
    // 2. lock basset
    // 3. Call rebalance again, with a new state
    Ok(Response::new()
        .add_messages(vec![
            WasmMsg::Execute {
                contract_addr: config.basset_token.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Send {
                    contract: config.anchor_custody_basset_contract.to_string(),
                    amount: basset_in_contract_address,
                    msg: to_binary(&AnchorCustodyCw20Msg::DepositCollateral {})?,
                })?,
                funds: vec![],
            },
            WasmMsg::Execute {
                contract_addr: config.anchor_overseer_contract.to_string(),
                msg: to_binary(&AnchorOverseerMsg::LockCollateral {
                    collaterals: vec![(config.basset_token.to_string(), basset_in_contract_address.into())],
                })?,
                funds: vec![],
            },
            WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::Anyone {
                    anyone_msg: AnyoneMsg::Rebalance {},
                })?,
                funds: vec![],
            },
        ])
        .add_attributes(vec![
            ("action", "deposit_all"),
            ("amount", &basset_in_contract_address.to_string()),
        ]))
}

pub(crate) fn withdraw_all_logic(
    deps: DepsMut,
    env: Env,
    config: &Config,
) -> StdResult<Response> {
    let aterra_balance = query_token_balance(deps.as_ref(), &config.aterra_token, &env.contract.address);

    let basset_in_custody = get_basset_in_custody(
        deps.as_ref(),
        &config.anchor_custody_basset_contract,
        &env.contract.address,
    )?;

    // 1. rebalance with basset_to_withdraw = basset_in_custody,
    //    that will sell aterra and repay loan
    // 2. unlock basset from anchor_overseer
    // 3. withdraw basset from anchor_custody

    let rebalance_response = rebalance(
        deps,
        env,
        &config,
        basset_in_custody,
        Some(basset_in_custody),
    )?;

    let response = rebalance_response
        .add_submessage(
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.anchor_overseer_contract.to_string(),
                msg: to_binary(&AnchorOverseerMsg::UnlockCollateral {
                    collaterals: vec![(config.basset_token.to_string(), basset_in_custody)],
                })?,
                funds: vec![],
            }))
        )
        .add_submessage(
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.anchor_custody_basset_contract.to_string(),
                msg: to_binary(&AnchorCustodyMsg::WithdrawCollateral {
                    amount: Some(basset_in_custody),
                })?,
                funds: vec![],
            }))
        )
        .add_attributes(vec![
            ("action", "withdraw_all"),
            ("amount", &aterra_balance.to_string()),
        ]);

    Ok(response)
}

/// Anyone can execute claim_anc_rewards function to claim
/// ANC rewards, swap ANC => UST token, swap
/// part of UST => PSI token and distribute
/// result PSI token to gov contract
pub fn claim_anc_rewards(deps: DepsMut, env: Env) -> StdResult<Response> {
    let config: Config = load_config(deps.storage)?;
    let last_rewards_claiming_height = load_last_rewards_claiming_height(deps.as_ref().storage)?;
    let current_height = env.block.height;

    if !is_anc_rewards_claimable(
        current_height,
        last_rewards_claiming_height,
        config.claiming_rewards_delay,
    ) {
        return Err(StdError::generic_err("claiming too often"));
    }

    store_last_rewards_claiming_height(deps.storage, &current_height)?;

    Ok(Response::new()
        .add_messages(vec![
            WasmMsg::Execute {
                contract_addr: config.anchor_market_contract.to_string(),
                msg: to_binary(&AnchorMarketMsg::ClaimRewards { to: None })?,
                funds: vec![],
            },
            WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::Yourself {
                    yourself_msg: YourselfMsg::SwapAnc {},
                })?,
                funds: vec![],
            },
        ])
        .add_attribute("action", "claim_anc_rewards"))
}

pub fn swap_anc(deps: DepsMut, env: Env) -> StdResult<Response> {
    let config: Config = load_config(deps.storage)?;

    let anc_amount =
        query_token_balance(deps.as_ref(), &config.anchor_token, &env.contract.address);

    if anc_amount.is_zero() {
        return Err(StdError::generic_err("ANC amount is zero").into());
    }

    let stable_coin_balance = query_balance(
        &deps.querier,
        &env.contract.address,
        config.stable_denom.clone(),
    )?;
    store_stable_balance_before_selling_anc(deps.storage, &stable_coin_balance)?;

    Ok(Response::new()
        .add_messages(vec![
            WasmMsg::Execute {
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
                funds: vec![],
            },
            WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::Yourself {
                    yourself_msg: YourselfMsg::DisributeRewards {},
                })?,
                funds: vec![],
            },
        ])
        .add_attributes(vec![
            ("action", "swap_anc"),
            ("anc_swapped", &anc_amount.to_string()),
        ]))
}

pub fn distribute_rewards(deps: DepsMut, env: Env) -> StdResult<Response> {
    let config: Config = load_config(deps.storage)?;
    let stable_coin_balance_before_sell_anc =
        load_stable_balance_before_selling_anc(deps.as_ref().storage)?;

    let stable_coin_balance = query_balance(
        &deps.querier,
        &env.contract.address,
        config.stable_denom.clone(),
    )?;
    let aterra_balance =
        query_token_balance(deps.as_ref(), &config.aterra_token, &env.contract.address);

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

pub fn claim_remainded_stables(deps: Deps, env: Env) -> StdResult<Response> {
    let config: Config = load_config(deps.storage)?;
    let borrower_info: BorrowerInfoResponse =
        query_borrower_info(deps, &config.anchor_market_contract, &env.contract.address)?;
    let borrowed_amount = borrower_info.loan_amount;

    if !borrowed_amount.is_zero() {
        Err(StdError::generic_err(format!(
            "wait until there will be 0 loan amount (no bAsset stakers), current loan: {}",
            borrowed_amount
        )))
    } else {
        let aterra_balance = query_token_balance(deps, &config.aterra_token, &env.contract.address);

        if aterra_balance.is_zero() {
            buy_psi_on_remainded_stable_coins(deps, env, config)
        } else {
            //Always because Anchor can block withdrawing
            //if there are too many borrowers
            Ok(Response::new()
                .add_submessage(SubMsg::reply_always(
                    WasmMsg::Execute {
                        contract_addr: config.aterra_token.to_string(),
                        msg: to_binary(&Cw20ExecuteMsg::Send {
                            contract: config.anchor_market_contract.to_string(),
                            amount: aterra_balance,
                            msg: to_binary(&AnchorMarketCw20Msg::RedeemStable {})?,
                        })?,
                        funds: vec![],
                    },
                    SubmsgIds::RedeemStableOnRemainder.id(),
                ))
                .add_attributes(vec![
                    ("action", "distribute_remainded_rewards"),
                    ("selling_aterra", &aterra_balance.to_string()),
                ]))
        }
    }
}

/// spend all stables we have, buy PSI and send it to Governance contract.
/// To governance directly because there is no nAsset in that moment, so
/// no reason to send PSI tokens to nAsset_staker (through psi_distributor)
pub fn buy_psi_on_remainded_stable_coins(
    deps: Deps,
    env: Env,
    config: Config,
) -> StdResult<Response> {
    let stable_coin_balance = query_balance(
        &deps.querier,
        &env.contract.address,
        config.stable_denom.clone(),
    )?;

    if stable_coin_balance.is_zero() {
        Ok(Response::default())
    } else {
        let tax_info = get_tax_info(deps, &config.stable_denom)?;
        let stable_coin_to_buy_psi: Uint128 =
            tax_info.subtract_tax(stable_coin_balance.into()).into();
        let swap_asset = Asset {
            info: AssetInfo::NativeToken {
                denom: config.stable_denom.clone(),
            },
            amount: stable_coin_to_buy_psi,
        };

        Ok(Response::new()
            .add_message(WasmMsg::Execute {
                contract_addr: config.psi_stable_swap_contract.to_string(),
                msg: to_binary(&TerraswapExecuteMsg::Swap {
                    offer_asset: swap_asset,
                    max_spread: None,
                    belief_price: None,
                    to: Some(config.governance_contract.to_string()),
                })?,
                funds: vec![Coin {
                    denom: config.stable_denom.clone(),
                    amount: stable_coin_to_buy_psi,
                }],
            })
            .add_attributes(vec![
                ("action", "distribute_remainded_rewards"),
                ("bying_psi", &stable_coin_to_buy_psi.to_string()),
            ]))
    }
}

fn get_time(block: &BlockInfo) -> u64 {
    block.time.seconds()
}
