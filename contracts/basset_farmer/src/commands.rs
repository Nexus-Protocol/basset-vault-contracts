use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Addr, Binary, CanonicalAddr, Coin, ContractInfo,
    CosmosMsg, Decimal, Deps, DepsMut, Empty, Env, MessageInfo, QueryRequest, Reply, ReplyOn,
    Response, StdError, StdResult, SubMsg, Uint128, WasmMsg, WasmQuery,
};
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::Cw20HookMsg as TerraswapCw20HookMsg;
use terraswap::pair::ExecuteMsg as TerraswapExecuteMsg;

use crate::{
    commands,
    contract::{SUBMSG_ID_FAKE_NO_REPLY, SUBMSG_ID_REDEEM_STABLE},
    queries,
    state::{
        load_config, load_farmer_info, load_repaying_loan_state, load_state, store_farmer_info,
        store_repaying_loan_state, FarmerInfo, RepayingLoanState, State,
    },
    utils::{calc_aterra_redeem_error_handling_action, calculate_aterra_amount_to_sell},
};
use crate::{error::ContractError, response::MsgInstantiateContractResponse};
use crate::{state::Config, ContractResult};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cw20::Cw20ReceiveMsg;
use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;
use yield_optimizer::{
    basset_farmer::{
        AnyoneMsg, Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, YourselfMsg,
    },
    basset_farmer_config::{
        query_borrower_action, BorrowerActionResponse, ConfigResponse as FarmerConfigConfigResponse,
    },
    get_tax_info,
    querier::{
        get_basset_in_custody, query_aterra_state, query_balance, query_borrower_info,
        query_supply, query_token_balance, AnchorMarketCw20Msg, AnchorMarketMsg,
        BorrowerInfoResponse,
    },
};

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> ContractResult<Response> {
    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Deposit) => commands::receive_cw20_deposit(deps, env, info, cw20_msg),
        //TODO: withdraw should work straigthforward, without cw20
        Ok(Cw20HookMsg::Withdraw) => commands::receive_cw20_withdraw(deps, env, info, cw20_msg),
        Err(err) => Err(ContractError::Std(err)),
    }
}

pub fn receive_cw20_deposit(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> ContractResult<Response> {
    let basset_addr = info.sender;
    // only basset contract can execute this message
    let config: Config = load_config(deps.storage)?;
    if basset_addr != config.basset_token {
        return Err(ContractError::Unauthorized {});
    }

    //we trust cw20 contract
    let farmer_addr: Addr = Addr::unchecked(cw20_msg.sender);

    receive_basset(deps, farmer_addr, cw20_msg.amount.into())
}

pub fn receive_basset(deps: DepsMut, farmer: Addr, amount: Uint256) -> ContractResult<Response> {
    let mut farmer_info: FarmerInfo = load_farmer_info(deps.storage, &farmer)?;

    farmer_info.spendable_basset += amount;

    store_farmer_info(deps.storage, &farmer, &farmer_info)?;

    Ok(Response {
        messages: vec![],
        submessages: vec![],
        attributes: vec![
            attr("action", "deposit_basset_step_1"),
            attr("farmer", farmer.as_str()),
            attr("amount", amount.to_string()),
        ],
        data: None,
    })
}

pub fn receive_cw20_withdraw(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> ContractResult<Response> {
    let contract_addr = info.sender;
    // only basset contract can execute this message
    let config: Config = load_config(deps.storage)?;
    if contract_addr != config.basset_token {
        return Err(ContractError::Unauthorized {});
    }

    //we trust cw20 contract
    let farmer_addr: Addr = Addr::unchecked(cw20_msg.sender);

    withdrawn_basset(deps, farmer_addr, cw20_msg.amount.into())
}

pub fn withdrawn_basset(deps: DepsMut, farmer: Addr, amount: Uint256) -> ContractResult<Response> {
    //TODO

    todo!();

    Ok(Response {
        messages: vec![],
        submessages: vec![],
        attributes: vec![],
        data: None,
    })
}

/// Executor: anyone
pub fn rebalance(deps: DepsMut, env: Env, info: MessageInfo) -> ContractResult<Response> {
    let config: Config = load_config(deps.storage)?;

    // basset balance in custody contract
    let basset_in_custody = get_basset_in_custody(
        deps.as_ref(),
        config.custody_basset_contract,
        env.contract.address.clone(),
    )?;

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
            //TODO: is it the best choice to return error here?
            return Err(StdError::generic_err("no rebalance needed").into());
        }
        BorrowerActionResponse::Borrow {
            amount,
            advised_buffer_size,
        } => {
            // borrow_logic(config, amount, advised_buffer_size);
            todo!()
        }
        BorrowerActionResponse::Repay {
            amount,
            advised_buffer_size,
        } => {
            let config = load_config(deps.storage)?;
            let state = load_state(deps.storage)?;
            repay_logic(deps, config, state, amount, advised_buffer_size)
        }
    }
}

fn borrow_logic(
    contract: ContractInfo,
    config: Config,
    borrow_amount: Uint256,
    buffer_size: Uint256,
) -> ContractResult<Response> {
    //TODO: handle 95% borrow error (use submessages)
    //cause if flow comes from Rebalance - you will try to Borrow again
    //on next Rebalance iteration...
    //
    //TODO: handle stable taxes - how much you will receive if Borrow Xust?
    Ok(Response {
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.anchor_market_contract.to_string(),
                msg: to_binary(&AnchorMarketMsg::BorrowStable {
                    borrow_amount: todo!(),
                    to: None,
                })?,
                send: vec![],
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract.address.to_string(),
                msg: to_binary(&YourselfMsg::AfterBorrow {
                    borrowed_amount: borrow_amount,
                    buffer_size,
                })?,
                send: vec![],
            }),
        ],
        submessages: vec![],
        attributes: vec![
            attr("action", "borrow_stable"),
            attr("amount", borrow_amount),
            attr("buffer_size", buffer_size),
        ],
        data: None,
    })
}

fn repay_logic(
    deps: DepsMut,
    config: Config,
    state: State,
    repay_amount: Uint256,
    aim_buffer_size: Uint256,
) -> ContractResult<Response> {
    //TODO: handle stable taxes - how much you will repay if Send Xust?

    let aterra_exchange_rate: Decimal256 =
        query_aterra_state(deps.as_ref(), &config.anchor_market_contract)?.exchange_rate;
    let aterra_amount_to_sell = calculate_aterra_amount_to_sell(
        &state,
        aterra_exchange_rate,
        repay_amount,
        aim_buffer_size,
    );

    let repaying_loan_state = RepayingLoanState {
        iteration_index: 0,
        aterra_amount_to_sell,
        aterra_amount_in_selling: aterra_amount_to_sell,
        aim_buffer_size,
    };
    store_repaying_loan_state(deps.storage, &repaying_loan_state)?;

    Ok(Response {
        //TODO: looks like we do not need this
        // messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
        //     contract_addr: contract.address.to_string(),
        //     msg: to_binary(&YourselfMsg::AfterAterraRedeem { repay_amount })?,
        //     send: vec![],
        // })],
        messages: vec![],
        submessages: vec![SubMsg {
            msg: WasmMsg::Execute {
                contract_addr: config.aterra_token.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Send {
                    contract: config.anchor_market_contract.to_string(),
                    amount: aterra_amount_to_sell.into(),
                    msg: to_binary(&AnchorMarketCw20Msg::RedeemStable {})?,
                })?,
                send: vec![],
            }
            .into(),
            gas_limit: None,
            id: SUBMSG_ID_REDEEM_STABLE,
            reply_on: ReplyOn::Always,
        }],
        attributes: vec![
            attr("action", "start_repaying_loan"),
            attr("aterra_amount_to_sell", aterra_amount_to_sell),
            attr("aim_buffer_size", aim_buffer_size),
        ],
        data: None,
    })
}

pub fn repay_reply_logic(deps: DepsMut, env: Env, on_success: bool) -> ContractResult<Response> {
    let mut repaying_loan_state = load_repaying_loan_state(deps.storage)?;
    //TODO: move '5' to const or config. Think about value. I think it is good idea to have
    //some limit or iteration, cause of crazy gas price, but not sure about the value.
    //to think
    if repaying_loan_state.iteration_index >= 5 {
        return Ok(Response::default());
    }

    if on_success {
        repaying_loan_state.aterra_amount_to_sell = repaying_loan_state.aterra_amount_to_sell
            - repaying_loan_state.aterra_amount_in_selling;
    }

    let config = load_config(deps.storage)?;
    let current_buffer_balance: Uint256 = query_balance(
        &deps.querier,
        &env.contract.address,
        config.stable_denom.to_string(),
    )?
    .into();

    let aterra_exchange_rate: Decimal256 =
        query_aterra_state(deps.as_ref(), &config.anchor_market_contract)?.exchange_rate;
    let tax_info = get_tax_info(deps.as_ref(), &config.stable_denom)?;
    let action_to_do = calc_aterra_redeem_error_handling_action(
        repaying_loan_state.aterra_amount_to_sell,
        aterra_exchange_rate,
        current_buffer_balance,
        repaying_loan_state.aim_buffer_size,
        &tax_info,
    );

    if let Some(action_to_do) = action_to_do {
        let mut submessages = Vec::with_capacity(2);

        let repay_stable_coin = Coin {
            denom: config.stable_denom.to_string(),
            amount: action_to_do.repay_loan_amount.into(),
        };
        submessages.push(SubMsg {
            //first message is to repay loan
            msg: WasmMsg::Execute {
                contract_addr: config.anchor_market_contract.to_string(),
                msg: to_binary(&AnchorMarketMsg::RepayStable {})?,
                send: vec![repay_stable_coin],
            }
            .into(),
            gas_limit: None,
            id: SUBMSG_ID_FAKE_NO_REPLY,
            reply_on: ReplyOn::Success,
        });

        if action_to_do.aterra_amount_to_sell > Uint256::zero() {
            submessages.push(SubMsg {
                //second message is to redeem stables again, but only for the
                //amount that was repayed
                msg: WasmMsg::Execute {
                    contract_addr: config.aterra_token.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Send {
                        contract: config.anchor_market_contract.to_string(),
                        amount: action_to_do.aterra_amount_to_sell.into(),
                        msg: to_binary(&AnchorMarketCw20Msg::RedeemStable {})?,
                    })?,
                    send: vec![],
                }
                .into(),
                gas_limit: None,
                id: SUBMSG_ID_REDEEM_STABLE,
                //do not reply on error, have no idea what to do in that case
                reply_on: ReplyOn::Success,
            });
        }

        repaying_loan_state.aterra_amount_in_selling = action_to_do.aterra_amount_to_sell;
        repaying_loan_state.iteration_index += 1;
        store_repaying_loan_state(deps.storage, &repaying_loan_state)?;

        Ok(Response {
            messages: vec![],
            submessages,
            attributes: vec![
                attr("action", "repay_loan_from_buffer"),
                attr("loan_repayment_amount", action_to_do.repay_loan_amount),
                attr("aterra_to_sell", action_to_do.aterra_amount_to_sell),
            ],
            data: None,
        })
    } else {
        return Ok(Response::default());
    }
}

/// Executor: overseer
pub fn deposit_basset(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    //TODO: we trust Overseer, so this should be Address
    farmer: String,
    deposit_amount: Uint256,
) -> ContractResult<Response> {
    let config: Config = load_config(deps.storage)?;
    if info.sender != config.overseer_contract {
        return Err(ContractError::Unauthorized {});
    }

    let farmer_addr: Addr = deps.api.addr_validate(&farmer)?;
    let mut farmer_info: FarmerInfo = load_farmer_info(deps.storage, &farmer_addr)?;
    if deposit_amount > farmer_info.spendable_basset {
        return Err(StdError::generic_err(format!(
            "Deposit amount cannot excceed the user's spendable amount: {}",
            farmer_info.spendable_basset
        ))
        .into());
    }

    // total cAsset supply
    let casset_supply: Uint256 = query_supply(&deps.querier, config.casset_token.clone())?.into();

    // basset balance in custody contract
    let basset_in_custody = get_basset_in_custody(
        deps.as_ref(),
        config.custody_basset_contract,
        env.contract.address.clone(),
    )?;

    // basset balance in cw20 contract
    let bluna_in_contract_address =
        query_token_balance(deps.as_ref(), &config.basset_token, &env.contract.address)?;

    // cAsset tokens to mint:
    // user_share = (deposited_basset / total_basset)
    // cAsset_to_mint = cAsset_supply * user_share / (1 - user_share)
    let basset_balance: Uint256 = basset_in_custody + bluna_in_contract_address.into();
    if basset_balance == Uint256::zero() {
        //impossible because if 'farmer' have 'spendable_basset' then he deposit some bAsset
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

    farmer_info.spendable_basset = farmer_info.spendable_basset - deposit_amount;
    farmer_info.balance_casset += casset_to_mint;
    store_farmer_info(deps.storage, &farmer_addr, &farmer_info)?;

    Ok(Response {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.casset_token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: farmer.clone(),
                //TODO: what is the reason to use Uint256 if we convert it to Uint128 at the end?
                amount: casset_to_mint.into(),
            })?,
            send: vec![],
        })],
        submessages: vec![],
        attributes: vec![
            attr("action", "deposit_basset_step_2"),
            attr("farmer", farmer),
            attr("amount", deposit_amount),
        ],
        data: None,
    })
}

/// Anyone can execute sweep function to claim
/// ANC rewards, swap ANC => UST token, swap
/// part of UST => PSI token and distribute
/// result PSI token to gov contract
pub fn sweep(deps: DepsMut, env: Env, info: MessageInfo) -> ContractResult<Response> {
    let config: Config = load_config(deps.storage)?;
    //TODO: should we care about Authorization here?

    Ok(Response {
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.anchor_market_contract.to_string(),
                msg: to_binary(&AnchorMarketMsg::ClaimRewards { to: None })?,
                send: vec![],
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::Anyone {
                    anyone_msg: AnyoneMsg::SwapAnc {},
                })?,
                send: vec![],
            }),
        ],
        submessages: vec![],
        attributes: vec![attr("action", "sweep")],
        data: None,
    })

    //1. claim ANC rewards
    //2. sell all ANC to UST
    //3. 95% is a rewards, calculate them, add to rewards. Update global_reward_index
    //4. 5% is for Psi stakers, swap UST to Psi and send them to Governance contract.
}

pub fn swap_anc(deps: DepsMut, env: Env, info: MessageInfo) -> ContractResult<Response> {
    let config: Config = load_config(deps.storage)?;
    //TODO: should we care about Authorization here?

    let amount = query_token_balance(deps.as_ref(), &config.anchor_token, &env.contract.address)?;
    Ok(Response {
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.anchor_token.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Send {
                    amount,
                    contract: config.anchor_ust_swap_contract.to_string(),
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
                msg: to_binary(&ExecuteMsg::Anyone {
                    anyone_msg: AnyoneMsg::DisributeRewards {},
                })?,
                send: vec![],
            }),
        ],
        submessages: vec![],
        attributes: vec![
            attr("action", "swap_anc"),
            attr("anc_swapped", format!("{:?}", amount.to_string())),
        ],
        data: None,
    })
}

//TODO: move stable denom to config?
const STABLE_DENOM: &str = "uust";

pub fn buy_psi_tokens(deps: DepsMut, env: Env, info: MessageInfo) -> ContractResult<Response> {
    let config: Config = load_config(deps.storage)?;
    //TODO: should we care about Authorization here?
    let ust_balance = query_balance(
        &deps.querier,
        &env.contract.address,
        STABLE_DENOM.to_string(),
    )?;

    //TODO: subtract UST buffer balance!
    let ust_to_buy_psi = ust_balance * config.psi_part_in_rewards;

    let swap_asset = Asset {
        info: AssetInfo::NativeToken {
            denom: STABLE_DENOM.to_string(),
        },
        amount: ust_to_buy_psi,
    };

    // deduct tax first
    let ust_to_buy_psi = (swap_asset.deduct_tax(&deps.querier)?).amount;

    Ok(Response {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.ust_psi_swap_contract.to_string(),
            msg: to_binary(&TerraswapExecuteMsg::Swap {
                offer_asset: Asset {
                    amount: ust_to_buy_psi,
                    ..swap_asset
                },
                max_spread: None,
                belief_price: None,
                to: None,
            })?,
            send: vec![Coin {
                denom: STABLE_DENOM.to_string(),
                amount: ust_to_buy_psi,
            }],
        })],
        submessages: vec![],
        attributes: vec![
            attr("action", "buy_psi_tokens"),
            attr("ust_spent", format!("{:?}", ust_to_buy_psi.to_string())),
        ],
        data: None,
    })
}

pub fn distribute_rewards(deps: DepsMut, env: Env, info: MessageInfo) -> ContractResult<Response> {
    //TODO: should we care about Authorization here?

    let config: Config = load_config(deps.storage)?;
    let ust_balance = query_balance(
        &deps.querier,
        &env.contract.address,
        STABLE_DENOM.to_string(),
    )?;
    let send_asset = Asset {
        info: AssetInfo::NativeToken {
            denom: STABLE_DENOM.to_string(),
        },
        amount: ust_balance,
    };
    let ust_to_deposit = (send_asset.deduct_tax(&deps.querier)?).amount;

    let psi_balance = query_token_balance(deps.as_ref(), &config.psi_token, &env.contract.address)?;
    Ok(Response {
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.anchor_market_contract.to_string(),
                msg: to_binary(&AnchorMarketMsg::DepositStable {})?,
                send: vec![Coin {
                    denom: STABLE_DENOM.to_string(),
                    amount: ust_to_deposit,
                }],
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.psi_token.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: config.governance_contract.to_string(),
                    amount: psi_balance,
                })?,
                send: vec![],
            }),
        ],
        submessages: vec![],
        attributes: vec![
            attr("action", "distribute_rewards"),
            attr("ust_to_deposit", ust_to_deposit),
            attr("psi_to_governance", psi_balance),
        ],
        data: None,
    })
}

pub fn claim_rewards(deps: DepsMut, env: Env, info: MessageInfo) -> ContractResult<Response> {
    //TODO: what if user sent his cAsset to someone? How we can manage rewards here?

    // 1. ask cAsset contract for user balance
    // 2. ask governance contract for user balance
    // 3. now you know his cAsset balance - calculate rewards based on diff between borrowed UST
    //    and UST in Anchor deposit
    todo!()
}
