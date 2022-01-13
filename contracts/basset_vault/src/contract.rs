use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply,
    Response, StdError, StdResult, SubMsg, WasmMsg,
};

use crate::reply_response::MsgInstantiateContractResponse;
use crate::state::{
    load_psi_distributor_init_info, store_psi_distributor_init_info, LegacyConfig, Config, PsiDistributorInitInfo,
};
use crate::{
    commands, queries,
    state::{
        config_set_nasset_token, config_set_psi_distributor, load_child_contracts_info,
        load_legacy_config, load_config, load_nasset_token_config_holder, store_child_contracts_info, store_config,
        store_nasset_token_config_holder, update_loan_state_part_of_loan_repaid,
        ChildContractsInfo,
    },
    SubmsgIds, TOO_HIGH_BORROW_DEMAND_ERR_MSG,
};
use basset_vault::{
    anchor::basset_custody::get_basset_in_custody,
    basset_vault::{
        AnyoneMsg, ExecuteMsg, GovernanceMsg, InstantiateMsg, MigrateMsg, QueryMsg, YourselfMsg,
    },
    nasset_token::InstantiateMsg as NAssetTokenInstantiateMsg,
    nasset_token_config_holder::{
        AnyoneMsg as NAssetTokenConfigHolderAnyoneMsg,
        ExecuteMsg as NAssetTokenConfigHolderExecuteMsg,
        InstantiateMsg as NAssetTokenConfigHolderInstantiateMsg,
    },
    nasset_token_rewards::InstantiateMsg as NAssetTokenRewardsInstantiateMsg,
    psi_distributor::InstantiateMsg as PsiDistributorInstantiateMsg,
    terraswap::AssetInfo,
    terraswap_factory::ExecuteMsg as TerraswapFactoryExecuteMsg,
};
use cw20::MinterResponse;
use protobuf::Message;
use std::convert::TryFrom;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        governance_contract: deps.api.addr_validate(&msg.gov_addr)?,
        anchor_token: deps.api.addr_validate(&msg.anchor_addr)?,
        anchor_overseer_contract: deps.api.addr_validate(&msg.a_overseer_addr)?,
        anchor_market_contract: deps.api.addr_validate(&msg.a_market_addr)?,
        anchor_custody_basset_contract: deps.api.addr_validate(&msg.a_custody_basset_addr)?,
        anchor_basset_reward_contract: deps.api.addr_validate(&msg.a_basset_reward_addr)?,
        anc_stable_swap_contract: deps.api.addr_validate(&msg.anc_stable_swap_addr)?,
        psi_stable_swap_contract: deps.api.addr_validate(&msg.psi_stable_swap_addr)?,
        basset_token: deps.api.addr_validate(&msg.basset_addr)?,
        aterra_token: deps.api.addr_validate(&msg.aterra_addr)?,
        psi_token: deps.api.addr_validate(&msg.psi_addr)?,
        basset_vault_strategy_contract: deps.api.addr_validate(&msg.basset_vs_addr)?,
        stable_denom: msg.stable_denom,
        claiming_rewards_delay: msg.claiming_rewards_delay,
        over_loan_balance_value: msg.over_loan_balance_value,
        nasset_token: Addr::unchecked(""),
        psi_distributor: Addr::unchecked(""),
    };
    store_config(deps.storage, &config)?;

    let child_contracts_info = ChildContractsInfo {
        nasset_token_code_id: msg.nasset_t_ci,
        nasset_token_rewards_code_id: msg.nasset_t_r_ci,
        psi_distributor_code_id: msg.psi_distr_ci,
        collateral_token_symbol: msg.collateral_ts,
        community_pool_contract_addr: msg.community_addr,
        manual_ltv: msg.manual_ltv,
        fee_rate: msg.fee_rate,
        tax_rate: msg.tax_rate,
    };
    store_child_contracts_info(deps.storage, &child_contracts_info)?;

    let psi_distributor_init_info = PsiDistributorInitInfo {
        terraswap_factory_contract_addr: msg.ts_factory_addr,
        // It will be set later, after pair creation
        nasset_psi_swap_contract_addr: None,
    };
    store_psi_distributor_init_info(deps.storage, &psi_distributor_init_info)?;

    Ok(Response::new().add_submessage(SubMsg::reply_on_success(
        CosmosMsg::Wasm(WasmMsg::Instantiate {
            admin: Some(config.governance_contract.to_string()),
            code_id: msg.nasset_t_ch_ci,
            msg: to_binary(&NAssetTokenConfigHolderInstantiateMsg {
                governance_contract_addr: msg.gov_addr,
            })?,
            funds: vec![],
            label: "".to_string(),
        }),
        SubmsgIds::InitNAssetConfigHolder.id(),
    )))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    let submessage_enum = SubmsgIds::try_from(msg.id)?;
    match submessage_enum {
        SubmsgIds::InitNAssetConfigHolder => {
            let data = msg.result.unwrap().data.unwrap();
            let res: MsgInstantiateContractResponse = Message::parse_from_bytes(data.as_slice())
                .map_err(|_| {
                    StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
                })?;

            let nasset_token_config_holder = res.get_contract_address();
            store_nasset_token_config_holder(
                deps.storage,
                &Addr::unchecked(nasset_token_config_holder),
            )?;
            let child_contracts_info = load_child_contracts_info(deps.as_ref().storage)?;
            let config = load_config(deps.storage)?;

            Ok(Response::new()
                .add_submessage(SubMsg::reply_on_success(
                    CosmosMsg::Wasm(WasmMsg::Instantiate {
                        admin: Some(config.governance_contract.to_string()),
                        code_id: child_contracts_info.nasset_token_code_id,
                        msg: to_binary(&NAssetTokenInstantiateMsg {
                            name: format!(
                                "Nexus b{} token share representation",
                                child_contracts_info.collateral_token_symbol
                            ),
                            symbol: format!("n{}", child_contracts_info.collateral_token_symbol),
                            decimals: 6,
                            initial_balances: vec![],
                            mint: Some(MinterResponse {
                                minter: env.contract.address.to_string(),
                                cap: None,
                            }),
                            marketing: None,
                            config_holder_contract: nasset_token_config_holder.to_string(),
                        })?,
                        funds: vec![],
                        label: "".to_string(),
                    }),
                    SubmsgIds::InitNAsset.id(),
                ))
                .add_attributes(vec![
                    ("action", "nasset_token_config_holder_initialized"),
                    (
                        "nasset_token_config_holder_addr",
                        nasset_token_config_holder,
                    ),
                ]))
        }

        SubmsgIds::InitNAsset => {
            let data = msg.result.unwrap().data.unwrap();
            let res: MsgInstantiateContractResponse = Message::parse_from_bytes(data.as_slice())
                .map_err(|_| {
                    StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
                })?;

            let nasset_token = res.get_contract_address();
            let config =
                config_set_nasset_token(deps.storage, deps.api.addr_validate(nasset_token)?)?;
            let psi_distributor_init_info = load_psi_distributor_init_info(deps.storage)?;

            Ok(Response::new()
                .add_submessage(SubMsg::reply_always(
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: psi_distributor_init_info.terraswap_factory_contract_addr,
                        msg: to_binary(&TerraswapFactoryExecuteMsg::CreatePair {
                            asset_infos: [
                                AssetInfo::Token {
                                    contract_addr: config.nasset_token,
                                },
                                AssetInfo::Token {
                                    contract_addr: config.psi_token,
                                },
                            ],
                        })?,
                        funds: vec![],
                    }),
                    SubmsgIds::InitNAssetPsiSwapPair.id(),
                ))
                .add_attributes(vec![
                    ("action", "nasset_token_initialized"),
                    ("nasset_token_addr", nasset_token),
                ]))
        }

        SubmsgIds::InitNAssetPsiSwapPair => {
            let events = msg
                .result
                .into_result()
                .map_err(|err| {
                    StdError::generic_err(format!("Error creating nAsset <-> Psi pair: {}", err))
                })?
                .events;

            let nasset_psi_swap_contract_addr = events
                .into_iter()
                .map(|event| event.attributes)
                .flatten()
                .find(|attr| attr.key == "pair_contract_addr")
                .map(|attr| attr.value)
                .ok_or_else(|| {
                    StdError::generic_err("Failed to create nAsset <-> Psi swap pair")
                })?;

            let config = load_config(deps.storage)?;
            let child_contracts_info = load_child_contracts_info(deps.as_ref().storage)?;
            let mut psi_distributor_init_info = load_psi_distributor_init_info(deps.storage)?;
            psi_distributor_init_info.nasset_psi_swap_contract_addr =
                Some(nasset_psi_swap_contract_addr);
            store_psi_distributor_init_info(deps.storage, &psi_distributor_init_info)?;

            Ok(Response::new()
                .add_submessage(SubMsg::reply_on_success(
                    CosmosMsg::Wasm(WasmMsg::Instantiate {
                        admin: Some(config.governance_contract.to_string()),
                        code_id: child_contracts_info.nasset_token_rewards_code_id,
                        msg: to_binary(&NAssetTokenRewardsInstantiateMsg {
                            psi_token_addr: config.psi_token.to_string(),
                            nasset_token_addr: config.nasset_token.to_string(),
                            governance_contract_addr: config.governance_contract.to_string(),
                        })?,
                        funds: vec![],
                        label: "".to_string(),
                    }),
                    SubmsgIds::InitNAssetRewards.id(),
                ))
                .add_attributes(vec![
                    ("action", "nasset_psi_swap_pair_initialized"),
                    (
                        "nasset_psi_swap_contract_addr",
                        &psi_distributor_init_info.into_nasset_psi_swap_contract_addr()?,
                    ),
                ]))
        }

        SubmsgIds::InitNAssetRewards => {
            let data = msg.result.unwrap().data.unwrap();
            let res: MsgInstantiateContractResponse = Message::parse_from_bytes(data.as_slice())
                .map_err(|_| {
                    StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
                })?;

            //we do not need to save nasset_token_rewards addr, cause there is no direct interactions
            let nasset_token_rewards = res.get_contract_address();
            let child_contracts_info = load_child_contracts_info(deps.as_ref().storage)?;
            let nasset_token_config_holder = load_nasset_token_config_holder(deps.storage)?;
            let psi_distributor_init_info = load_psi_distributor_init_info(deps.storage)?;
            let config = load_config(deps.storage)?;

            Ok(Response::new()
                .add_submessage(SubMsg::reply_on_success(
                    CosmosMsg::Wasm(WasmMsg::Instantiate {
                        admin: Some(config.governance_contract.to_string()),
                        code_id: child_contracts_info.psi_distributor_code_id,
                        msg: to_binary(&PsiDistributorInstantiateMsg {
                            psi_token_addr: config.psi_token.to_string(),
                            nasset_token_rewards_contract_addr: nasset_token_rewards.to_string(),
                            governance_contract_addr: config.governance_contract.to_string(),
                            basset_vault_strategy_contract_addr: config
                                .basset_vault_strategy_contract
                                .to_string(),
                            nasset_psi_swap_contract_addr: psi_distributor_init_info
                                .into_nasset_psi_swap_contract_addr()?,
                            community_pool_contract_addr: child_contracts_info
                                .community_pool_contract_addr,
                            manual_ltv: child_contracts_info.manual_ltv,
                            fee_rate: child_contracts_info.fee_rate,
                            tax_rate: child_contracts_info.tax_rate,
                        })?,
                        funds: vec![],
                        label: "".to_string(),
                    }),
                    SubmsgIds::InitPsiDistributor.id(),
                ))
                .add_message(WasmMsg::Execute {
                    contract_addr: nasset_token_config_holder.to_string(),
                    funds: vec![],
                    msg: to_binary(&NAssetTokenConfigHolderExecuteMsg::Anyone {
                        anyone_msg: NAssetTokenConfigHolderAnyoneMsg::SetTokenRewardsContract {
                            nasset_token_rewards_contract_addr: nasset_token_rewards.to_string(),
                        },
                    })?,
                })
                .add_attributes(vec![
                    ("action", "nasset_token_rewards_initialized"),
                    ("nasset_token_rewards_addr", nasset_token_rewards),
                ]))
        }

        SubmsgIds::InitPsiDistributor => {
            let data = msg.result.unwrap().data.unwrap();
            let res: MsgInstantiateContractResponse = Message::parse_from_bytes(data.as_slice())
                .map_err(|_| {
                    StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
                })?;

            let psi_distributor = res.get_contract_address();
            config_set_psi_distributor(deps.storage, deps.api.addr_validate(psi_distributor)?)?;

            Ok(Response::new().add_attributes(vec![
                ("action", "psi_distributor_initialized"),
                ("psi_distributor_addr", psi_distributor),
            ]))
        }

        SubmsgIds::RedeemStableOnRepayLoan => match msg.result {
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

        SubmsgIds::RepayLoan => {
            let _ = update_loan_state_part_of_loan_repaid(deps.storage)?;
            Ok(Response::default())
        }

        SubmsgIds::Borrowing => commands::borrow_logic_on_reply(deps, env),

        SubmsgIds::RedeemStableOnRemainder => {
            let config = load_config(deps.storage)?;
            //we do not care about errors here, just send all your stables to governance
            commands::buy_psi_on_remainded_stable_coins(deps.as_ref(), env, config)
        }

        SubmsgIds::HoldingReward => commands::holding_reward_logic_on_reply(deps, env),
    }
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => commands::receive_cw20(deps, env, info, msg),

        ExecuteMsg::Anyone { anyone_msg } => match anyone_msg {
            AnyoneMsg::Rebalance {} => {
                let config = load_config(deps.storage)?;

                // basset balance in custody contract
                let basset_in_custody = get_basset_in_custody(
                    deps.as_ref(),
                    &config.anchor_custody_basset_contract,
                    &env.contract.address.clone(),
                )?;

                commands::rebalance(deps, env, &config, basset_in_custody, None)
            }

            AnyoneMsg::HonestWork {} => commands::claim_reward(deps, env),

            AnyoneMsg::ClaimRemainder {} => commands::claim_remainded_stables(deps.as_ref(), env),

            AnyoneMsg::AcceptGovernance {} => commands::accept_governance(deps, env, info),
        },

        ExecuteMsg::Yourself { yourself_msg } => {
            if info.sender != env.contract.address {
                return Err(StdError::generic_err("unauthorized"));
            }

            match yourself_msg {
                YourselfMsg::SwapAnc {} => commands::swap_anc(deps, env),
                YourselfMsg::DisributeRewards {} => commands::distribute_rewards(deps, env),
            }
        }

        ExecuteMsg::Governance { governance_msg } => {
            let config: Config = load_config(deps.storage)?;
            if info.sender != config.governance_contract {
                return Err(StdError::generic_err("unauthorized"));
            }

            match governance_msg {
                GovernanceMsg::UpdateConfig {
                    psi_distributor_addr,
                    anchor_overseer_contract_addr,
                    anchor_market_contract_addr,
                    anchor_custody_basset_contract_addr,
                    anchor_basset_reward_addr,
                    anc_stable_swap_contract_addr,
                    psi_stable_swap_contract_addr,
                    basset_vault_strategy_contract_addr,
                    claiming_rewards_delay,
                    over_loan_balance_value,
                } => commands::update_config(
                    deps,
                    config,
                    psi_distributor_addr,
                    anchor_overseer_contract_addr,
                    anchor_market_contract_addr,
                    anchor_custody_basset_contract_addr,
                    anchor_basset_reward_addr,
                    anc_stable_swap_contract_addr,
                    psi_stable_swap_contract_addr,
                    basset_vault_strategy_contract_addr,
                    claiming_rewards_delay,
                    over_loan_balance_value,
                ),

                GovernanceMsg::UpdateGovernanceContract {
                    gov_addr,
                    seconds_to_wait_for_accept_gov_tx,
                } => commands::update_governance_addr(
                    deps,
                    env,
                    gov_addr,
                    seconds_to_wait_for_accept_gov_tx,
                ),
            }
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
        QueryMsg::Rebalance {} => to_binary(&queries::query_rebalance(deps, env)?),
        QueryMsg::ChildContractsCodeId {} => to_binary(&queries::child_contracts_code_id(deps)?),
        QueryMsg::IsRewardsClaimable {} => to_binary(&queries::is_rewards_claimable(deps, env)?),
    }
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> StdResult<Response> {
    let legacy_config: LegacyConfig = load_legacy_config(deps.storage)?;
    
    let new_config = Config {
        governance_contract: legacy_config.governance_contract,
        anchor_token: legacy_config.anchor_token,
        anchor_overseer_contract: legacy_config.anchor_overseer_contract,
        anchor_market_contract: legacy_config.anchor_market_contract,
        anchor_custody_basset_contract: legacy_config.anchor_custody_basset_contract,
        anchor_basset_reward_contract: deps.api.addr_validate(&msg.anchor_basset_reward_addr)?,
        anc_stable_swap_contract: legacy_config.anc_stable_swap_contract,
        psi_stable_swap_contract: legacy_config.psi_stable_swap_contract,
        basset_token: legacy_config.basset_token,
        aterra_token: legacy_config.aterra_token,
        psi_token: legacy_config.psi_token,
        basset_vault_strategy_contract: legacy_config.basset_vault_strategy_contract,
        stable_denom: legacy_config.stable_denom,
        claiming_rewards_delay: legacy_config.claiming_rewards_delay,
        over_loan_balance_value: legacy_config.over_loan_balance_value,
        nasset_token: legacy_config.nasset_token,
        psi_distributor: legacy_config.psi_distributor,
    };
    
    store_config(deps.storage, &new_config)?;
    Ok(Response::default())
}
