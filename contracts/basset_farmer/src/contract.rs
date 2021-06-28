use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{
    attr, entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn,
    Response, StdError, StdResult, SubMsg, WasmMsg,
};

use crate::{
    commands, queries,
    state::{
        config_set_nasset_token, config_set_psi_distributor, load_child_contracts_code_id,
        load_config, store_child_contracts_code_id, store_config,
        update_loan_state_part_of_loan_repaid, ChildContractsCodeId,
    },
};
use crate::{error::ContractError, response::MsgInstantiateContractResponse};
use crate::{state::Config, ContractResult};
use cw20::MinterResponse;
use cw20_base::msg::InstantiateMsg as TokenInstantiateMsg;
use protobuf::Message;
use std::str::FromStr;
use yield_optimizer::{
    basset_farmer::{AnyoneMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, YourselfMsg},
    nasset_staker::InstantiateMsg as NAssetStakerInstantiateMsg,
    psi_distributor::InstantiateMsg as PsiDistributorInstantiateMsg,
    querier::get_basset_in_custody,
};

pub const SUBMSG_ID_INIT_NASSET: u64 = 1;
pub const SUBMSG_ID_REDEEM_STABLE: u64 = 2;
pub const SUBMSG_ID_REPAY_LOAN: u64 = 3;
pub const SUBMSG_ID_BORROWING: u64 = 4;
pub const SUBMSG_ID_INIT_NASSET_STAKER: u64 = 5;
pub const SUBMSG_ID_INIT_PSI_DISTRIBUTOR: u64 = 6;
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
        nasset_token: Addr::unchecked(""),
        basset_token: deps.api.addr_validate(&msg.basset_token_addr)?,
        anchor_custody_basset_contract: deps
            .api
            .addr_validate(&msg.anchor_custody_basset_contract)?,
        governance_contract: deps.api.addr_validate(&msg.governance_addr)?,
        anchor_token: deps.api.addr_validate(&msg.anchor_token)?,
        anchor_overseer_contract: deps.api.addr_validate(&msg.anchor_overseer_contract)?,
        anchor_market_contract: deps.api.addr_validate(&msg.anchor_market_contract)?,
        anc_stable_swap_contract: deps.api.addr_validate(&msg.anc_stable_swap_contract)?,
        psi_stable_swap_contract: deps.api.addr_validate(&msg.psi_stable_swap_contract)?,
        aterra_token: deps.api.addr_validate(&msg.aterra_token)?,
        psi_token: deps.api.addr_validate(&msg.psi_token)?,
        basset_farmer_config_contract: deps
            .api
            .addr_validate(&msg.basset_farmer_config_contract)?,
        stable_denom: msg.stable_denom,
        claiming_rewards_delay: msg.claiming_rewards_delay,
        psi_distributor_addr: Addr::unchecked(""),
        over_loan_balance_value: Decimal256::from_str(&msg.over_loan_balance_value)?,
    };
    store_config(deps.storage, &config)?;

    let child_contracts_code_id = ChildContractsCodeId {
        nasset_token: msg.nasset_token_code_id,
        nasset_staker: msg.nasset_staker_code_id,
        psi_distributor: msg.psi_distributor_code_id,
    };
    store_child_contracts_code_id(deps.storage, &child_contracts_code_id)?;

    Ok(Response {
        messages: vec![],
        submessages: vec![SubMsg {
            msg: WasmMsg::Instantiate {
                admin: None,
                code_id: msg.nasset_token_code_id,
                msg: to_binary(&TokenInstantiateMsg {
                    name: "nexus basset token share representation".to_string(),
                    symbol: format!("n{}", msg.collateral_token_symbol),
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
            id: SUBMSG_ID_INIT_NASSET,
            reply_on: ReplyOn::Success,
        }],
        attributes: vec![attr("action", "initialization")],
        data: None,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> ContractResult<Response> {
    match msg.id {
        SUBMSG_ID_INIT_NASSET => {
            let data = msg.result.unwrap().data.unwrap();
            let res: MsgInstantiateContractResponse = Message::parse_from_bytes(data.as_slice())
                .map_err(|_| {
                    StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
                })?;

            let nasset_token = res.get_contract_address();
            config_set_nasset_token(deps.storage, deps.api.addr_validate(nasset_token)?)?;
            let child_contracts_code_id = load_child_contracts_code_id(deps.as_ref().storage)?;
            let config = load_config(deps.storage)?;

            Ok(Response {
                messages: vec![],
                submessages: vec![SubMsg {
                    msg: WasmMsg::Instantiate {
                        admin: None,
                        code_id: child_contracts_code_id.nasset_staker,
                        msg: to_binary(&NAssetStakerInstantiateMsg {
                            nasset_token: nasset_token.to_string(),
                            psi_token: config.psi_token.to_string(),
                            governance_contract: config.governance_contract.to_string(),
                        })?,
                        send: vec![],
                        label: "".to_string(),
                    }
                    .into(),
                    gas_limit: None,
                    id: SUBMSG_ID_INIT_NASSET_STAKER,
                    reply_on: ReplyOn::Success,
                }],
                attributes: vec![
                    attr("action", "nasset_token_initialized"),
                    attr("nasset_token_addr", nasset_token),
                ],
                data: None,
            })
        }

        SUBMSG_ID_INIT_NASSET_STAKER => {
            let data = msg.result.unwrap().data.unwrap();
            let res: MsgInstantiateContractResponse = Message::parse_from_bytes(data.as_slice())
                .map_err(|_| {
                    StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
                })?;

            let nasset_staker = res.get_contract_address();
            let config = load_config(deps.as_ref().storage)?;
            let child_contracts_code_id = load_child_contracts_code_id(deps.as_ref().storage)?;
            //we do not need to save nasset_staker addr here, cause there is no direct interactions

            Ok(Response {
                messages: vec![],
                submessages: vec![SubMsg {
                    msg: WasmMsg::Instantiate {
                        admin: None,
                        code_id: child_contracts_code_id.psi_distributor,
                        msg: to_binary(&PsiDistributorInstantiateMsg {
                            nasset_token_contract: config.nasset_token.to_string(),
                            nasset_staker_contract: nasset_staker.to_string(),
                            governance_contract: config.governance_contract.to_string(),
                        })?,
                        send: vec![],
                        label: "".to_string(),
                    }
                    .into(),
                    gas_limit: None,
                    id: SUBMSG_ID_INIT_PSI_DISTRIBUTOR,
                    reply_on: ReplyOn::Success,
                }],
                attributes: vec![
                    attr("action", "nasset_staker_initialized"),
                    attr("nasset_staker_addr", nasset_staker),
                ],
                data: None,
            })
        }

        SUBMSG_ID_INIT_PSI_DISTRIBUTOR => {
            let data = msg.result.unwrap().data.unwrap();
            let res: MsgInstantiateContractResponse = Message::parse_from_bytes(data.as_slice())
                .map_err(|_| {
                    StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
                })?;

            let psi_distributor = res.get_contract_address();
            config_set_psi_distributor(deps.storage, deps.api.addr_validate(psi_distributor)?)?;

            Ok(Response {
                messages: vec![],
                submessages: vec![],
                attributes: vec![
                    attr("action", "psi_distributor_initialized"),
                    attr("psi_distributor_addr", psi_distributor),
                ],
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
                    &config.anchor_custody_basset_contract,
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
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config => to_binary(&queries::query_config(deps)?),
        QueryMsg::Rebalance => to_binary(&queries::query_rebalance(deps, env)?),
        QueryMsg::ChildContractsCodeId => to_binary(&queries::child_contracts_code_id(deps)?),
        QueryMsg::IsRewardsClaimable => to_binary(&queries::is_rewards_claimable(deps, env)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResult<Response> {
    Ok(Response::default())
}
