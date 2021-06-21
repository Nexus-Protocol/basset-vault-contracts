use commands::repay_logic_on_reply;
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Addr, Binary, CanonicalAddr, Coin, CosmosMsg, Deps,
    DepsMut, Env, MessageInfo, Reply, ReplyOn, Response, StdError, StdResult, SubMsg, Uint128,
    WasmMsg,
};

use crate::{
    commands, queries,
    state::{
        config_set_casset_staker, config_set_casset_token, load_aim_buffer_size,
        load_casset_staking_code_id, load_config, load_repaying_loan_state,
        remove_casset_staking_code_id, store_casset_staking_code_id, store_config,
        update_loan_state_part_of_loan_repaid, RepayingLoanState,
    },
};
use crate::{error::ContractError, response::MsgInstantiateContractResponse};
use crate::{state::Config, ContractResult};
use cw20::{Cw20ReceiveMsg, MinterResponse};
use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;
use cw20_base::msg::InstantiateMsg as TokenInstantiateMsg;
use protobuf::Message;
use yield_optimizer::{
    basset_farmer::{
        AnyoneMsg, CAssetStakerMsg, Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
        YourselfMsg,
    },
    casset_staking::InstantiateMsg as CAssetStakerInstantiateMsg,
    deduct_tax, get_tax_info,
    querier::{
        get_basset_in_custody, query_aterra_state, query_balance, AnchorMarketCw20Msg,
        AnchorMarketMsg,
    },
};

pub const SUBMSG_ID_INIT_CASSET: u64 = 1;
pub const SUBMSG_ID_REDEEM_STABLE: u64 = 2;
pub const SUBMSG_ID_REPAY_LOAN: u64 = 3;
pub const SUBMSG_ID_BORROWING: u64 = 4;
pub const SUBMSG_ID_INIT_CASSET_STAKER: u64 = 5;
//withdrawing from Anchor Deposit error
pub const TOO_HIGH_BORROW_DEMAND_ERR_MSG: &str = "borrow demand too high";
//borrowing error
pub const TOO_HIGH_BORROW_AMOUNT_ERR_MSG: &str = "borrow amount too high";

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    let config = Config {
        casset_token: Addr::unchecked(""),
        casset_staking_contract: Addr::unchecked(""),
        basset_token: deps.api.addr_validate(&msg.basset_token_addr)?,
        custody_basset_contract: deps.api.addr_validate(&msg.custody_basset_contract)?,
        governance_contract: deps.api.addr_validate(&msg.governance_addr)?,
        anchor_token: deps.api.addr_validate(&msg.anchor_token)?,
        anchor_overseer_contract: deps.api.addr_validate(&msg.anchor_overseer_contract)?,
        anchor_market_contract: deps.api.addr_validate(&msg.anchor_market_contract)?,
        anchor_ust_swap_contract: deps.api.addr_validate(&msg.anchor_ust_swap_contract)?,
        ust_psi_swap_contract: deps.api.addr_validate(&msg.ust_psi_swap_contract)?,
        aterra_token: deps.api.addr_validate(&msg.aterra_token)?,
        psi_part_in_rewards: msg.psi_part_in_rewards,
        psi_token: deps.api.addr_validate(&msg.psi_token)?,
        basset_farmer_config_contract: deps
            .api
            .addr_validate(&msg.basset_farmer_config_contract)?,
        stable_denom: msg.stable_denom,
    };
    store_config(deps.storage, &config)?;

    store_casset_staking_code_id(deps.storage, &msg.casset_staking_code_id)?;

    Ok(Response {
        messages: vec![],
        submessages: vec![SubMsg {
            msg: WasmMsg::Instantiate {
                admin: None,
                code_id: msg.token_code_id,
                msg: to_binary(&TokenInstantiateMsg {
                    name: "nexus basset token share representation".to_string(),
                    symbol: format!("c{}", msg.collateral_token_symbol),
                    decimals: 6,
                    initial_balances: vec![],
                    mint: Some(MinterResponse {
                        minter: env.contract.address.to_string(),
                        cap: None,
                    }),
                })?,
                send: vec![],
                label: "".to_string(),
            }
            .into(),
            gas_limit: None,
            id: SUBMSG_ID_INIT_CASSET,
            reply_on: ReplyOn::Success,
        }],
        attributes: vec![],
        data: None,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> ContractResult<Response> {
    match msg.id {
        SUBMSG_ID_INIT_CASSET => {
            let data = msg.result.unwrap().data.unwrap();
            let res: MsgInstantiateContractResponse = Message::parse_from_bytes(data.as_slice())
                .map_err(|_| {
                    StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
                })?;

            let casset_token = res.get_contract_address();
            config_set_casset_token(deps.storage, deps.api.addr_validate(casset_token)?)?;
            let config = load_config(deps.as_ref().storage)?;
            let casset_staking_code_id = load_casset_staking_code_id(deps.as_ref().storage)?;
            remove_casset_staking_code_id(deps.storage);

            Ok(Response {
                messages: vec![],
                submessages: vec![SubMsg {
                    msg: WasmMsg::Instantiate {
                        admin: None,
                        code_id: casset_staking_code_id,
                        msg: to_binary(&CAssetStakerInstantiateMsg {
                            casset_token: casset_token.to_string(),
                            aterra_token: config.aterra_token.to_string(),
                            stable_denom: config.stable_denom,
                            basset_farmer_contract: env.contract.address.to_string(),
                            anchor_market_contract: config.anchor_market_contract.to_string(),
                        })?,
                        send: vec![],
                        label: "".to_string(),
                    }
                    .into(),
                    gas_limit: None,
                    id: SUBMSG_ID_INIT_CASSET_STAKER,
                    reply_on: ReplyOn::Success,
                }],
                attributes: vec![attr("casset_token_addr", casset_token)],
                data: None,
            })
        }

        SUBMSG_ID_INIT_CASSET_STAKER => {
            let data = msg.result.unwrap().data.unwrap();
            let res: MsgInstantiateContractResponse = Message::parse_from_bytes(data.as_slice())
                .map_err(|_| {
                    StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
                })?;

            let casset_staker = res.get_contract_address();
            config_set_casset_staker(deps.storage, deps.api.addr_validate(casset_staker)?)?;

            Ok(Response {
                messages: vec![],
                submessages: vec![],
                attributes: vec![attr("casset_staking_contract", casset_staker)],
                data: None,
            })
        }

        SUBMSG_ID_REDEEM_STABLE => match msg.result {
            cosmwasm_std::ContractResult::Err(err_msg) => {
                if err_msg
                    .to_lowercase()
                    .contains(TOO_HIGH_BORROW_DEMAND_ERR_MSG)
                {
                    //we need to repay loan a bit, before redeem stables
                    commands::repay_logic_on_reply(deps, env)
                } else {
                    return Err(StdError::generic_err(format!(
                        "fail to redeem stables, reason: {}",
                        err_msg
                    ))
                    .into());
                }
            }
            cosmwasm_std::ContractResult::Ok(_) => commands::repay_logic_on_reply(deps, env),
        },

        SUBMSG_ID_REPAY_LOAN => {
            let _ = update_loan_state_part_of_loan_repaid(deps.storage)?;
            Ok(Response::default())
        }

        SUBMSG_ID_BORROWING => commands::borrow_logic_on_reply(deps, env),

        unknown => {
            Err(StdError::generic_err(format!("unknown reply message id: {}", unknown)).into())
        }
    }
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => commands::receive_cw20(deps, env, info, msg),

        ExecuteMsg::Anyone { anyone_msg } => match anyone_msg {
            AnyoneMsg::Rebalance => {
                let config: Config = load_config(deps.storage)?;

                // basset balance in custody contract
                let basset_in_custody = get_basset_in_custody(
                    deps.as_ref(),
                    &config.custody_basset_contract,
                    &env.contract.address.clone(),
                )?;

                commands::rebalance(deps, env, &config, basset_in_custody, None)
            }

            AnyoneMsg::HonestWork => commands::claim_anc_rewards(deps, env),
        },

        ExecuteMsg::Yourself { yourself_msg } => {
            if info.sender != env.contract.address {
                return Err(ContractError::Unauthorized {});
            }

            match yourself_msg {
                YourselfMsg::SwapAnc => commands::swap_anc(deps, env),
                YourselfMsg::DisributeRewards => commands::distribute_rewards(deps, env),
            }
        }

        ExecuteMsg::CAssetStaker { casset_staker_msg } => {
            let config: Config = load_config(deps.storage)?;
            if info.sender != config.casset_staking_contract {
                return Err(ContractError::Unauthorized {});
            }

            match casset_staker_msg {
                CAssetStakerMsg::SendRewards { recipient, amount } => {
                    commands::send_rewards(deps, env, config, recipient, amount)
                }
            }
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config => to_binary(&queries::query_config(deps)?),
        QueryMsg::Rebalance => to_binary(&queries::query_rebalance(deps, env)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResult<Response> {
    Ok(Response::default())
}
