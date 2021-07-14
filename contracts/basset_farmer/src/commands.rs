use cosmwasm_std::{
    attr, from_binary, to_binary, Addr, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, ReplyOn,
    Response, StdError, SubMsg, Uint128, WasmMsg,
};

use crate::{
    commands,
    state::{
        load_aim_buffer_size, load_config, load_repaying_loan_state,
        load_stable_balance_before_selling_anc, query_external_config, query_external_config_light,
        store_aim_buffer_size, store_config, store_last_rewards_claiming_height,
        store_repaying_loan_state, store_stable_balance_before_selling_anc, RepayingLoanState,
    },
    tax_querier::get_tax_info,
    utils::{
        calc_after_borrow_action, get_repay_loan_action, is_anc_rewards_claimable,
        split_profit_to_handle_interest,
    },
    SubmsgIds,
};
use crate::{error::ContractError, state::load_last_rewards_claiming_height};
use crate::{state::Config, ContractResult};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use yield_optimizer::basset_farmer_config_holder::Config as ExternalConfig;
use yield_optimizer::{
    basset_farmer::{AnyoneMsg, Cw20HookMsg, ExecuteMsg, YourselfMsg},
    basset_farmer_strategy::{query_borrower_action, BorrowerActionResponse},
    querier::{
        get_basset_in_custody, query_aterra_state, query_balance, query_borrower_info,
        query_supply, query_token_balance, AnchorMarketCw20Msg, AnchorMarketMsg, AnchorOverseerMsg,
        BorrowerInfoResponse,
    },
    terraswap::{Asset, AssetInfo},
    terraswap_pair::{Cw20HookMsg as TerraswapCw20HookMsg, ExecuteMsg as TerraswapExecuteMsg},
};

pub fn update_config(
    deps: DepsMut,
    mut current_config: Config,
    psi_distributor_addr: Option<String>,
) -> ContractResult<Response> {
    if let Some(ref psi_distributor_addr) = psi_distributor_addr {
        current_config.psi_distributor = deps.api.addr_validate(psi_distributor_addr)?;
    }

    store_config(deps.storage, &current_config)?;
    Ok(Response::default())
}

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
    let external_config: ExternalConfig = query_external_config_light(deps.as_ref(), &config)?;
    if basset_addr != external_config.basset_token {
        return Err(ContractError::Unauthorized);
    }

    //we trust cw20 contract
    let farmer_addr: Addr = Addr::unchecked(cw20_msg.sender);

    deposit_basset(
        deps,
        env,
        config,
        external_config,
        farmer_addr,
        cw20_msg.amount.into(),
    )
}

pub fn deposit_basset(
    deps: DepsMut,
    env: Env,
    config: Config,
    external_config: ExternalConfig,
    farmer: Addr,
    deposit_amount: Uint256,
) -> ContractResult<Response> {
    let nasset_supply: Uint256 = query_supply(&deps.querier, &config.nasset_token.clone())?.into();

    let basset_in_custody = get_basset_in_custody(
        deps.as_ref(),
        &external_config.anchor_custody_basset_contract,
        &env.contract.address,
    )?;

    if basset_in_custody.is_zero() && !nasset_supply.is_zero() {
        //read comments in 'withdraw_basset' function for a reason to return error here
        return Err(StdError::generic_err(
            "bAsset balance is zero, but nLuna supply is not! Freeze contract.",
        )
        .into());
    }

    // basset balance in cw20 contract
    let basset_in_contract_address = query_token_balance(
        deps.as_ref(),
        &external_config.basset_token,
        &env.contract.address,
    )?;

    let basset_balance: Uint256 = basset_in_custody + basset_in_contract_address.into();
    if basset_balance == Uint256::zero() {
        //impossible because 'farmer' already sent some basset
        return Err(ContractError::Impossible(
            "basset balance is zero".to_string(),
        ));
    }
    let farmer_basset_share: Decimal256 =
        Decimal256::from_ratio(deposit_amount.0, basset_balance.0);

    // nAsset tokens to mint:
    // user_share = (deposited_basset / total_basset)
    // nAsset_to_mint = nAsset_supply * user_share / (1 - user_share)
    let nasset_to_mint = if farmer_basset_share == Decimal256::one() {
        deposit_amount
    } else {
        // 'nasset_supply' can't be zero here, cause we already mint some for first farmer
        nasset_supply * farmer_basset_share / (Decimal256::one() - farmer_basset_share)
    };

    //1. lock basset
    //2. mint nasset
    //3. rebalance
    Ok(Response {
        messages: vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: external_config.anchor_overseer_contract.to_string(),
                msg: to_binary(&AnchorOverseerMsg::LockCollateral {
                    collaterals: vec![(external_config.basset_token.to_string(), deposit_amount)],
                })?,
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.nasset_token.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Mint {
                    recipient: farmer.to_string(),
                    amount: nasset_to_mint.into(),
                })?,
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::Anyone {
                    anyone_msg: AnyoneMsg::Rebalance,
                })?,
                funds: vec![],
            })),
        ],
        events: vec![],
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
    // only nAsset contract can execute this message
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
    nasset_to_withdraw_amount: Uint256,
) -> ContractResult<Response> {
    //nasset_to_withdraw_amount is not zero here, cw20 contract check it

    let external_config = query_external_config_light(deps.as_ref(), &config)?;
    //basset_in_contract_address is always zero (except Deposit stage)
    let basset_in_custody = get_basset_in_custody(
        deps.as_ref(),
        &external_config.anchor_custody_basset_contract,
        &env.contract.address,
    )?;

    let nasset_token_supply = query_supply(&deps.querier, &config.nasset_token)?;

    if basset_in_custody.is_zero() {
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
            "bAsset balance is zero, but nLuna supply is not! Freeze contract.",
        )
        .into());
    }

    let share_to_withdraw: Decimal256 = Decimal256::from_ratio(
        nasset_to_withdraw_amount.0,
        Uint256::from(nasset_token_supply).0,
    );
    let basset_to_withdraw: Uint256 = basset_in_custody * share_to_withdraw;

    //1. rebalance in a way you don't have basset_to_withdraw
    //2. unlock basset from custody
    //3. send basset to farmer
    //4. burn nasset
    let mut rebalance_response = rebalance(
        deps,
        env,
        &config,
        &external_config,
        basset_in_custody,
        Some(basset_to_withdraw),
    )?;

    rebalance_response
        .messages
        .push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: external_config.anchor_overseer_contract.to_string(),
            msg: to_binary(&AnchorOverseerMsg::UnlockCollateral {
                collaterals: vec![(external_config.basset_token.to_string(), basset_to_withdraw)],
            })?,
            funds: vec![],
        })));

    rebalance_response
        .messages
        .push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: external_config.basset_token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: farmer.to_string(),
                amount: basset_to_withdraw.into(),
            })?,
            funds: vec![],
        })));

    rebalance_response
        .messages
        .push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.nasset_token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Burn {
                amount: nasset_to_withdraw_amount.into(),
            })?,
            funds: vec![],
        })));

    rebalance_response
        .attributes
        .push(attr("action", "withdraw"));
    rebalance_response
        .attributes
        .push(attr("nasset_amount", nasset_to_withdraw_amount));

    Ok(rebalance_response)
}

/// Executor: anyone
pub fn rebalance(
    deps: DepsMut,
    env: Env,
    config: &Config,
    external_config: &ExternalConfig,
    basset_in_custody: Uint256,
    basset_to_withdraw: Option<Uint256>,
) -> ContractResult<Response> {
    let basset_in_custody = basset_in_custody - basset_to_withdraw.unwrap_or_default();

    let borrower_info: BorrowerInfoResponse = query_borrower_info(
        deps.as_ref(),
        &external_config.anchor_market_contract,
        &env.contract.address,
    )?;
    let borrowed_ust = borrower_info.loan_amount;

    let borrower_action = query_borrower_action(
        deps.as_ref(),
        &external_config.basset_farmer_strategy_contract,
        borrowed_ust,
        basset_in_custody,
    )?;

    match borrower_action {
        BorrowerActionResponse::Nothing => {
            //maybe it is better to return error here, but
            //we cant, cause it is used in 'withdraw'
            return Ok(Response {
                messages: vec![],
                events: vec![],
                attributes: vec![attr("action", "rebalance_not_needed")],
                data: None,
            });
        }

        BorrowerActionResponse::Borrow {
            amount,
            advised_buffer_size,
        } => {
            store_aim_buffer_size(deps.storage, &advised_buffer_size)?;
            borrow_logic(external_config, amount, advised_buffer_size)
        }

        BorrowerActionResponse::Repay {
            amount,
            advised_buffer_size,
        } => {
            store_aim_buffer_size(deps.storage, &advised_buffer_size)?;
            let mut repaying_loan_state = load_repaying_loan_state(deps.as_ref().storage)?;
            repaying_loan_state.to_repay_amount = amount;
            repaying_loan_state.aim_buffer_size = advised_buffer_size;
            repay_logic(deps, env, external_config, repaying_loan_state)
        }
    }
}

fn borrow_logic(
    external_config: &ExternalConfig,
    borrow_amount: Uint256,
    aim_buffer_size: Uint256,
) -> ContractResult<Response> {
    Ok(Response {
        events: vec![],
        messages: vec![SubMsg {
            msg: WasmMsg::Execute {
                contract_addr: external_config.anchor_market_contract.to_string(),
                msg: to_binary(&AnchorMarketMsg::BorrowStable {
                    borrow_amount,
                    to: None,
                })?,
                funds: vec![],
            }
            .into(),
            gas_limit: None,
            id: SubmsgIds::Borrowing.id(),
            // If can't borrow from Anchor we can't do anything, so just return error, consequence:
            // 1. user will not be able to deposit
            // 2. Rebalance return error
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
    let external_config = query_external_config(deps.as_ref())?;
    let tax_info = get_tax_info(deps.as_ref(), &external_config.stable_denom)?;
    let aim_buf_size = load_aim_buffer_size(deps.as_ref().storage)?;
    let stable_coin_balance = query_balance(
        &deps.querier,
        &env.contract.address,
        external_config.stable_denom.clone(),
    )?;
    let after_borrow_action =
        calc_after_borrow_action(stable_coin_balance.into(), aim_buf_size, &tax_info);
    after_borrow_action.to_response(&external_config)
}

pub(crate) fn repay_logic(
    deps: DepsMut,
    env: Env,
    external_config: &ExternalConfig,
    mut repaying_loan_state: RepayingLoanState,
) -> ContractResult<Response> {
    let aterra_balance = query_token_balance(
        deps.as_ref(),
        &external_config.aterra_token,
        &env.contract.address,
    )?;
    let aterra_exchange_rate: Decimal256 =
        query_aterra_state(deps.as_ref(), &external_config.anchor_market_contract)?.exchange_rate;
    let stable_coin_balance = query_balance(
        &deps.querier,
        &env.contract.address,
        external_config.stable_denom.clone(),
    )?;

    let tax_info = get_tax_info(deps.as_ref(), &external_config.stable_denom)?;
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

    repay_action.to_response(&external_config)
}

pub(crate) const LOAN_REPAYMENT_MAX_RECURSION_DEEP: u8 = 6;

pub(crate) fn repay_logic_on_reply(deps: DepsMut, env: Env) -> ContractResult<Response> {
    let mut repaying_loan_state = load_repaying_loan_state(deps.storage)?;
    repaying_loan_state.iteration_index += 1;
    if repaying_loan_state.iteration_index >= LOAN_REPAYMENT_MAX_RECURSION_DEEP {
        if repaying_loan_state.repayed_something {
            return Ok(Response::default());
        } else {
            return Err(StdError::generic_err("fail to repay loan").into());
        }
    }
    let external_config = query_external_config(deps.as_ref())?;
    repay_logic(deps, env, &external_config, repaying_loan_state)
}

/// Anyone can execute claim_anc_rewards function to claim
/// ANC rewards, swap ANC => UST token, swap
/// part of UST => PSI token and distribute
/// result PSI token to gov contract
pub fn claim_anc_rewards(deps: DepsMut, env: Env) -> ContractResult<Response> {
    let external_config: ExternalConfig = query_external_config(deps.as_ref())?;
    let last_rewards_claiming_height = load_last_rewards_claiming_height(deps.as_ref().storage)?;
    let current_height = env.block.height;

    if !is_anc_rewards_claimable(
        current_height,
        last_rewards_claiming_height,
        external_config.claiming_rewards_delay,
    ) {
        return Err(StdError::generic_err("claiming too often").into());
    }

    store_last_rewards_claiming_height(deps.storage, &current_height)?;

    Ok(Response {
        messages: vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: external_config.anchor_market_contract.to_string(),
                msg: to_binary(&AnchorMarketMsg::ClaimRewards { to: None })?,
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::Yourself {
                    yourself_msg: YourselfMsg::SwapAnc,
                })?,
                funds: vec![],
            })),
        ],
        events: vec![],
        attributes: vec![attr("action", "claim_anc_rewards")],
        data: None,
    })
}

pub fn swap_anc(deps: DepsMut, env: Env) -> ContractResult<Response> {
    let external_config: ExternalConfig = query_external_config(deps.as_ref())?;

    let stable_coin_balance = query_balance(
        &deps.querier,
        &env.contract.address,
        external_config.stable_denom.clone(),
    )?;
    store_stable_balance_before_selling_anc(deps.storage, &stable_coin_balance)?;

    let anc_amount = query_token_balance(
        deps.as_ref(),
        &external_config.anchor_token,
        &env.contract.address,
    )?;

    if anc_amount.is_zero() {
        return Err(StdError::generic_err("ANC amount is zero").into());
    }

    Ok(Response {
        messages: vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: external_config.anchor_token.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Send {
                    amount: anc_amount,
                    contract: external_config.anc_stable_swap_contract.to_string(),
                    msg: to_binary(&TerraswapCw20HookMsg::Swap {
                        belief_price: None,
                        max_spread: None,
                        to: None,
                    })?,
                })?,
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::Yourself {
                    yourself_msg: YourselfMsg::DisributeRewards,
                })?,
                funds: vec![],
            })),
        ],
        events: vec![],
        attributes: vec![attr("action", "swap_anc"), attr("anc_swapped", anc_amount)],
        data: None,
    })
}

pub fn distribute_rewards(deps: DepsMut, env: Env) -> ContractResult<Response> {
    let config: Config = load_config(deps.storage)?;
    let external_config: ExternalConfig = query_external_config_light(deps.as_ref(), &config)?;
    let stable_coin_balance_before_sell_anc =
        load_stable_balance_before_selling_anc(deps.as_ref().storage)?;

    let stable_coin_balance = query_balance(
        &deps.querier,
        &env.contract.address,
        external_config.stable_denom.clone(),
    )?;
    let aterra_balance = query_token_balance(
        deps.as_ref(),
        &external_config.aterra_token,
        &env.contract.address,
    )?;

    let aterra_state = query_aterra_state(deps.as_ref(), &external_config.anchor_market_contract)?;
    let borrower_info: BorrowerInfoResponse = query_borrower_info(
        deps.as_ref(),
        &external_config.anchor_market_contract,
        &env.contract.address,
    )?;
    let borrowed_amount = borrower_info.loan_amount;

    let action_with_profit = split_profit_to_handle_interest(
        borrowed_amount,
        aterra_balance.into(),
        aterra_state.exchange_rate,
        stable_coin_balance.into(),
        stable_coin_balance_before_sell_anc.into(),
        external_config.over_loan_balance_value,
    );

    let tax_info = get_tax_info(deps.as_ref(), &external_config.stable_denom)?;

    action_with_profit.to_response(&config, &external_config, &tax_info)
}

pub fn claim_remainded_stables(deps: Deps, env: Env) -> ContractResult<Response> {
    let external_config: ExternalConfig = query_external_config(deps)?;
    let borrower_info: BorrowerInfoResponse = query_borrower_info(
        deps,
        &external_config.anchor_market_contract,
        &env.contract.address,
    )?;
    let borrowed_amount = borrower_info.loan_amount;

    if !borrowed_amount.is_zero() {
        Err(StdError::generic_err(format!(
            "wait until there will be 0 loan amount (no bAsset stakers), current loan: {}",
            borrowed_amount
        ))
        .into())
    } else {
        let aterra_balance =
            query_token_balance(deps, &external_config.aterra_token, &env.contract.address)?;

        if aterra_balance.is_zero() {
            buy_psi_on_remainded_stable_coins(deps, env, external_config)
        } else {
            Ok(Response {
                events: vec![],
                messages: vec![SubMsg {
                    msg: WasmMsg::Execute {
                        contract_addr: external_config.aterra_token.to_string(),
                        msg: to_binary(&Cw20ExecuteMsg::Send {
                            contract: external_config.anchor_market_contract.to_string(),
                            amount: aterra_balance,
                            msg: to_binary(&AnchorMarketCw20Msg::RedeemStable {})?,
                        })?,
                        funds: vec![],
                    }
                    .into(),
                    gas_limit: None,
                    id: SubmsgIds::RedeemStableOnRemainder.id(),
                    //Always because Anchor can block withdrawing
                    //if there are too many borrowers
                    reply_on: ReplyOn::Always,
                }],
                attributes: vec![
                    attr("action", "distribute_remainded_rewards"),
                    attr("selling_aterra", aterra_balance),
                ],
                data: None,
            })
        }
    }
}

/// spend all stables we have, buy PSI and send it to Governance contract.
/// To governance directly because there is no nAsset in that moment, so
/// no reason to send PSI tokens to nAsset_staker (through psi_distributor)
pub fn buy_psi_on_remainded_stable_coins(
    deps: Deps,
    env: Env,
    external_config: ExternalConfig,
) -> ContractResult<Response> {
    let stable_coin_balance = query_balance(
        &deps.querier,
        &env.contract.address,
        external_config.stable_denom.clone(),
    )?;

    if stable_coin_balance.is_zero() {
        Ok(Response::default())
    } else {
        let tax_info = get_tax_info(deps, &external_config.stable_denom)?;
        let stable_coin_to_buy_psi: Uint128 =
            tax_info.subtract_tax(stable_coin_balance.into()).into();
        let swap_asset = Asset {
            info: AssetInfo::NativeToken {
                denom: external_config.stable_denom.clone(),
            },
            amount: stable_coin_to_buy_psi,
        };

        Ok(Response {
            messages: vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: external_config.psi_stable_swap_contract.to_string(),
                msg: to_binary(&TerraswapExecuteMsg::Swap {
                    offer_asset: swap_asset,
                    max_spread: None,
                    belief_price: None,
                    to: Some(external_config.governance_contract.to_string()),
                })?,
                funds: vec![Coin {
                    denom: external_config.stable_denom.clone(),
                    amount: stable_coin_to_buy_psi,
                }],
            }))],
            events: vec![],
            attributes: vec![
                attr("action", "distribute_remainded_rewards"),
                attr("bying_psi", stable_coin_to_buy_psi),
            ],
            data: None,
        })
    }
}
