use crate::{
    response::MsgInstantiateContractResponse,
    state::{load_config, Config},
    SubmsgIds,
};

use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::{
    attr,
    testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR},
    Addr, Api, CosmosMsg, OwnedDeps, Querier, Storage,
};
use cosmwasm_std::{to_binary, ContractResult, Reply, ReplyOn, SubMsg, SubcallResponse, WasmMsg};
use cw20::MinterResponse;
use protobuf::Message;
use std::str::FromStr;
use yield_optimizer::psi_distributor::InstantiateMsg as PsiDistributorInstantiateMsg;
use yield_optimizer::{
    nasset_token::InstantiateMsg as NAssetTokenInstantiateMsg,
    nasset_token_config_holder::{
        AnyoneMsg as NAssetTokenConfigHolderAnyoneMsg,
        ExecuteMsg as NAssetTokenConfigHolderExecuteMsg,
        InstantiateMsg as NAssetTokenConfigHolderInstantiateMsg,
    },
    nasset_token_rewards::InstantiateMsg as NAssetTokenRewardsInstantiateMsg,
};

#[test]
fn proper_initialization() {
    let nasset_contract_addr = "addr0001".to_string();
    let nasset_token_code_id = 10u64; //cw20 contract code
    let nasset_token_config_holder_code_id = 11u64;
    let nasset_token_rewards_code_id = 12u64; //contract code
    let psi_distributor_code_id = 13u64; //contract code
    let aterra_token = "addr0010".to_string();
    let stable_denom = "uust".to_string();
    let anchor_market_contract = "addr0007".to_string();
    let psi_distributor_contract = "addr0015".to_string();
    let governance_contract = "addr0016".to_string();
    let psi_token = "addr0011".to_string();
    let nasset_token_config_holder_contract = "addr0017".to_string();
    let nasset_token_rewards_contract = "addr0018".to_string();
    let over_loan_balance_value = "1.01".to_string();

    let msg = yield_optimizer::basset_farmer::InstantiateMsg {
        nasset_token_code_id,
        nasset_token_config_holder_code_id,
        nasset_token_rewards_code_id,
        psi_distributor_code_id,
        collateral_token_symbol: "Luna".to_string(),
        basset_token_addr: "addr0002".to_string(),
        anchor_custody_basset_contract: "addr0003".to_string(),
        anchor_overseer_contract: "addr0004".to_string(),
        governance_contract: governance_contract.clone(),
        anchor_token: "addr0006".to_string(),
        anchor_market_contract: anchor_market_contract.clone(),
        anc_stable_swap_contract: "addr0008".to_string(),
        psi_stable_swap_contract: "addr0009".to_string(),
        aterra_token: aterra_token.clone(),
        psi_token: psi_token.clone(),
        basset_farmer_config_contract: "addr0012".to_string(),
        stable_denom: stable_denom.clone(),
        claiming_rewards_delay: 1000,
        over_loan_balance_value: over_loan_balance_value.clone(),
    };
    let mut deps = mock_dependencies(&[]);
    instantiate_basset_farmer(
        &mut deps,
        msg.clone(),
        &psi_token,
        &nasset_token_config_holder_contract,
        &nasset_contract_addr,
        &nasset_token_rewards_contract,
        &psi_distributor_contract,
    );

    // ==========================================================
    // ================== CHECK FINAL STATE =====================
    // ==========================================================
    let config = load_config(deps.as_ref().storage).unwrap();
    assert_eq!(
        config,
        Config {
            nasset_token: Addr::unchecked(nasset_contract_addr.clone()),
            basset_token: Addr::unchecked(msg.basset_token_addr),
            anchor_custody_basset_contract: Addr::unchecked(msg.anchor_custody_basset_contract),
            governance_contract: Addr::unchecked(msg.governance_contract),
            anchor_token: Addr::unchecked(msg.anchor_token),
            anchor_overseer_contract: Addr::unchecked(msg.anchor_overseer_contract),
            anchor_market_contract: Addr::unchecked(msg.anchor_market_contract),
            anc_stable_swap_contract: Addr::unchecked(msg.anc_stable_swap_contract),
            psi_stable_swap_contract: Addr::unchecked(msg.psi_stable_swap_contract),
            aterra_token: Addr::unchecked(msg.aterra_token),
            psi_token: Addr::unchecked(msg.psi_token),
            basset_farmer_config_contract: Addr::unchecked(msg.basset_farmer_config_contract),
            stable_denom: msg.stable_denom.clone(),
            claiming_rewards_delay: msg.claiming_rewards_delay,
            psi_distributor_addr: Addr::unchecked(psi_distributor_contract.clone()),
            over_loan_balance_value: Decimal256::from_str(&msg.over_loan_balance_value).unwrap(),
        }
    );
}

pub fn instantiate_basset_farmer<A: Storage, B: Api, C: Querier>(
    deps: &mut OwnedDeps<A, B, C>,
    init_msg: yield_optimizer::basset_farmer::InstantiateMsg,
    psi_token: &str,
    nasset_token_config_holder_contract: &str,
    nasset_contract_addr: &str,
    nasset_token_rewards_contract: &str,
    psi_distributor_contract: &str,
) {
    let info = mock_info("addr0000", &[]);

    // ==========================================================
    // ================ Instantiate BASSET_FARMER ===============
    // ==========================================================
    {
        let res = crate::contract::instantiate(deps.as_mut(), mock_env(), info, init_msg.clone())
            .unwrap();
        assert_eq!(
            res.submessages,
            vec![SubMsg {
                msg: WasmMsg::Instantiate {
                    code_id: init_msg.nasset_token_config_holder_code_id,
                    msg: to_binary(&NAssetTokenConfigHolderInstantiateMsg {
                        governance_contract_addr: init_msg.governance_contract.clone()
                    })
                    .unwrap(),
                    send: vec![],
                    label: "".to_string(),
                    admin: None,
                }
                .into(),
                gas_limit: None,
                id: SubmsgIds::InitNAssetConfigHolder.id(),
                reply_on: ReplyOn::Success,
            }]
        );
    }

    // ==========================================================
    // ========= Instantiate NASSET_TOKEN_CONFIG_HOLDER =========
    // ==========================================================

    {
        let mut nasset_token_config_holder_initiate_response =
            MsgInstantiateContractResponse::new();
        nasset_token_config_holder_initiate_response
            .set_contract_address(nasset_token_config_holder_contract.to_string());

        let reply_msg = Reply {
            id: SubmsgIds::InitNAssetConfigHolder.id(),
            result: ContractResult::Ok(SubcallResponse {
                events: vec![],
                data: Some(
                    nasset_token_config_holder_initiate_response
                        .write_to_bytes()
                        .unwrap()
                        .into(),
                ),
            }),
        };

        let res = crate::contract::reply(deps.as_mut(), mock_env(), reply_msg.clone()).unwrap();
        assert_eq!(
            res.submessages,
            vec![SubMsg {
                msg: WasmMsg::Instantiate {
                    code_id: init_msg.nasset_token_code_id,
                    msg: to_binary(&NAssetTokenInstantiateMsg {
                        name: "nexus bAsset token share representation".to_string(),
                        symbol: "nLuna".to_string(),
                        decimals: 6,
                        initial_balances: vec![],
                        mint: Some(MinterResponse {
                            minter: MOCK_CONTRACT_ADDR.to_string(),
                            cap: None,
                        }),
                        config_holder_contract: nasset_token_config_holder_contract.to_string()
                    })
                    .unwrap(),
                    send: vec![],
                    label: "".to_string(),
                    admin: None,
                }
                .into(),
                gas_limit: None,
                id: SubmsgIds::InitNAsset.id(),
                reply_on: ReplyOn::Success,
            }]
        );

        assert_eq!(
            res.attributes,
            vec![
                attr("action", "nasset_token_config_holder_initialized"),
                attr(
                    "nasset_token_config_holder_addr",
                    nasset_token_config_holder_contract,
                ),
            ]
        );
    }

    // ==========================================================
    // ========= Instantiate NASSET_TOKEN =======================
    // ==========================================================
    {
        let mut nasset_token_initiate_response = MsgInstantiateContractResponse::new();
        nasset_token_initiate_response.set_contract_address(nasset_contract_addr.to_string());

        let reply_msg = Reply {
            id: SubmsgIds::InitNAsset.id(),
            result: ContractResult::Ok(SubcallResponse {
                events: vec![],
                data: Some(
                    nasset_token_initiate_response
                        .write_to_bytes()
                        .unwrap()
                        .into(),
                ),
            }),
        };

        let res = crate::contract::reply(deps.as_mut(), mock_env(), reply_msg.clone()).unwrap();
        assert_eq!(
            res.submessages,
            vec![SubMsg {
                msg: WasmMsg::Instantiate {
                    code_id: init_msg.nasset_token_rewards_code_id,
                    msg: to_binary(&NAssetTokenRewardsInstantiateMsg {
                        nasset_token_addr: nasset_contract_addr.to_string(),
                        psi_token_addr: init_msg.psi_token.clone(),
                        governance_contract_addr: init_msg.governance_contract.clone()
                    })
                    .unwrap(),
                    send: vec![],
                    label: "".to_string(),
                    admin: None,
                }
                .into(),
                gas_limit: None,
                id: SubmsgIds::InitNAssetRewards.id(),
                reply_on: ReplyOn::Success,
            }]
        );
        assert_eq!(
            res.attributes,
            vec![
                attr("action", "nasset_token_initialized"),
                attr("nasset_token_addr", nasset_contract_addr),
            ]
        );
    }

    // ==========================================================
    // ======== Set TOKEN_REWARDS_ADDR to CONFIG_HOLDER =========
    // ============= Instantiate PSI_DISTRIBUTOR ================
    // ==========================================================
    {
        let mut nasset_token_rewards_initiate_response = MsgInstantiateContractResponse::new();
        nasset_token_rewards_initiate_response
            .set_contract_address(nasset_token_rewards_contract.to_string());

        let reply_msg = Reply {
            id: SubmsgIds::InitNAssetRewards.id(),
            result: ContractResult::Ok(SubcallResponse {
                events: vec![],
                data: Some(
                    nasset_token_rewards_initiate_response
                        .write_to_bytes()
                        .unwrap()
                        .into(),
                ),
            }),
        };

        let res = crate::contract::reply(deps.as_mut(), mock_env(), reply_msg.clone()).unwrap();

        assert_eq!(
            res.messages,
            vec![CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: nasset_token_config_holder_contract.to_string(),
                send: vec![],
                msg: to_binary(&NAssetTokenConfigHolderExecuteMsg::Anyone {
                    anyone_msg: NAssetTokenConfigHolderAnyoneMsg::SetTokenRewardsContract {
                        nasset_token_rewards_contract_addr: nasset_token_rewards_contract
                            .to_string(),
                    },
                })
                .unwrap(),
            })]
        );

        assert_eq!(
            res.submessages,
            vec![SubMsg {
                msg: WasmMsg::Instantiate {
                    code_id: init_msg.psi_distributor_code_id,
                    msg: to_binary(&PsiDistributorInstantiateMsg {
                        psi_token_addr: psi_token.to_string(),
                        nasset_token_rewards_contract_addr: nasset_token_rewards_contract
                            .to_string(),
                        governance_contract_addr: init_msg.governance_contract.clone(),
                    })
                    .unwrap(),
                    send: vec![],
                    label: "".to_string(),
                    admin: None,
                }
                .into(),
                gas_limit: None,
                id: SubmsgIds::InitPsiDistributor.id(),
                reply_on: ReplyOn::Success,
            }]
        );
        assert_eq!(
            res.attributes,
            vec![
                attr("action", "nasset_token_rewards_initialized"),
                attr("nasset_token_rewards_addr", nasset_token_rewards_contract),
            ]
        );
    }

    // ==========================================================
    // ============= PSI_DISTRIBUTOR initialized ================
    // ==========================================================
    {
        let mut psi_distributor_initiate_response = MsgInstantiateContractResponse::new();
        psi_distributor_initiate_response
            .set_contract_address(psi_distributor_contract.to_string());

        let reply_msg = Reply {
            id: SubmsgIds::InitPsiDistributor.id(),
            result: ContractResult::Ok(SubcallResponse {
                events: vec![],
                data: Some(
                    psi_distributor_initiate_response
                        .write_to_bytes()
                        .unwrap()
                        .into(),
                ),
            }),
        };

        let res = crate::contract::reply(deps.as_mut(), mock_env(), reply_msg.clone()).unwrap();
        assert!(res.submessages.is_empty());
        assert!(res.messages.is_empty());
        assert_eq!(
            res.attributes,
            vec![
                attr("action", "psi_distributor_initialized"),
                attr("psi_distributor_addr", psi_distributor_contract),
            ]
        );
    }
}
