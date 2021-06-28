use crate::{
    contract::{
        SUBMSG_ID_INIT_NASSET, SUBMSG_ID_INIT_NASSET_STAKER, SUBMSG_ID_INIT_PSI_DISTRIBUTOR,
    },
    response::MsgInstantiateContractResponse,
    state::{load_config, Config},
};

use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::{
    attr,
    testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR},
    Addr,
};
use cosmwasm_std::{to_binary, ContractResult, Reply, ReplyOn, SubMsg, SubcallResponse, WasmMsg};
use cw20::MinterResponse;
use cw20_base::msg::InstantiateMsg as TokenInstantiateMsg;
use protobuf::Message;
use std::str::FromStr;
use yield_optimizer::nasset_staker::InstantiateMsg as NAssetStakerInstantiateMsg;
use yield_optimizer::psi_distributor::InstantiateMsg as PsiDistributorInstantiateMsg;

#[test]
fn proper_initialization() {
    let nluna_contract_addr = "addr0001".to_string();
    let nasset_token_code_id = 10u64; //cw20 contract code
    let nasset_staker_code_id = 11u64; //contract code
    let psi_distributor_code_id = 12u64; //contract code
    let aterra_token = "addr0010".to_string();
    let stable_denom = "uust".to_string();
    let anchor_market_contract = "addr0007".to_string();
    let nluna_staker_contract = "addr0014".to_string();
    let psi_distributor_contract = "addr0015".to_string();
    let governance_contract = "addr0016".to_string();
    let psi_token = "addr0011".to_string();
    let over_loan_balance_value = "1.01".to_string();
    let mut deps = mock_dependencies(&[]);

    let msg = yield_optimizer::basset_farmer::InstantiateMsg {
        nasset_token_code_id,
        nasset_staker_code_id,
        psi_distributor_code_id,
        collateral_token_symbol: "Luna".to_string(),
        basset_token_addr: "addr0002".to_string(),
        anchor_custody_basset_contract: "addr0003".to_string(),
        anchor_overseer_contract: "addr0004".to_string(),
        governance_addr: governance_contract.clone(),
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

    let info = mock_info("addr0000", &[]);
    let res = crate::contract::instantiate(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();
    assert_eq!(
        res.submessages,
        vec![SubMsg {
            msg: WasmMsg::Instantiate {
                code_id: nasset_token_code_id,
                msg: to_binary(&TokenInstantiateMsg {
                    name: "nexus basset token share representation".to_string(),
                    symbol: "nLuna".to_string(),
                    decimals: 6,
                    initial_balances: vec![],
                    mint: Some(MinterResponse {
                        minter: MOCK_CONTRACT_ADDR.to_string(),
                        cap: None,
                    }),
                })
                .unwrap(),
                send: vec![],
                label: "".to_string(),
                admin: None,
            }
            .into(),
            gas_limit: None,
            id: SUBMSG_ID_INIT_NASSET,
            reply_on: ReplyOn::Success,
        }]
    );
    assert_eq!(res.attributes, vec![attr("action", "initialization")]);

    let mut cw20_instantiate_response = MsgInstantiateContractResponse::new();
    cw20_instantiate_response.set_contract_address(nluna_contract_addr.clone());

    // store nLuna token address
    let reply_msg = Reply {
        id: SUBMSG_ID_INIT_NASSET,
        result: ContractResult::Ok(SubcallResponse {
            events: vec![],
            data: Some(cw20_instantiate_response.write_to_bytes().unwrap().into()),
        }),
    };

    let res = crate::contract::reply(deps.as_mut(), mock_env(), reply_msg.clone()).unwrap();
    assert_eq!(
        res.submessages,
        vec![SubMsg {
            msg: WasmMsg::Instantiate {
                code_id: nasset_staker_code_id,
                msg: to_binary(&NAssetStakerInstantiateMsg {
                    nasset_token: nluna_contract_addr.clone(),
                    psi_token: psi_token.clone(),
                    governance_contract: governance_contract.clone()
                })
                .unwrap(),
                send: vec![],
                label: "".to_string(),
                admin: None,
            }
            .into(),
            gas_limit: None,
            id: SUBMSG_ID_INIT_NASSET_STAKER,
            reply_on: ReplyOn::Success,
        }]
    );
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "nasset_token_initialized"),
            attr("nasset_token_addr", nluna_contract_addr.clone()),
        ]
    );

    let mut cw20_instantiate_response_2 = MsgInstantiateContractResponse::new();
    cw20_instantiate_response_2.set_contract_address(nluna_staker_contract.clone());
    // store psi_distributor contract address
    let reply_msg_2 = Reply {
        id: SUBMSG_ID_INIT_NASSET_STAKER,
        result: ContractResult::Ok(SubcallResponse {
            events: vec![],
            data: Some(cw20_instantiate_response_2.write_to_bytes().unwrap().into()),
        }),
    };
    let res = crate::contract::reply(deps.as_mut(), mock_env(), reply_msg_2.clone()).unwrap();
    assert_eq!(
        res.submessages,
        vec![SubMsg {
            msg: WasmMsg::Instantiate {
                code_id: psi_distributor_code_id,
                msg: to_binary(&PsiDistributorInstantiateMsg {
                    nasset_token_contract: nluna_contract_addr.clone(),
                    nasset_staker_contract: nluna_staker_contract.clone(),
                    governance_contract: governance_contract.clone(),
                })
                .unwrap(),
                send: vec![],
                label: "".to_string(),
                admin: None,
            }
            .into(),
            gas_limit: None,
            id: SUBMSG_ID_INIT_PSI_DISTRIBUTOR,
            reply_on: ReplyOn::Success,
        }]
    );
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "nasset_staker_initialized"),
            attr("nasset_staker_addr", nluna_staker_contract),
        ]
    );

    let mut cw20_instantiate_response_2 = MsgInstantiateContractResponse::new();
    cw20_instantiate_response_2.set_contract_address(psi_distributor_contract.clone());
    // store nasset_staker contract address
    let reply_msg_3 = Reply {
        id: SUBMSG_ID_INIT_PSI_DISTRIBUTOR,
        result: ContractResult::Ok(SubcallResponse {
            events: vec![],
            data: Some(cw20_instantiate_response_2.write_to_bytes().unwrap().into()),
        }),
    };
    let res = crate::contract::reply(deps.as_mut(), mock_env(), reply_msg_3.clone()).unwrap();
    assert!(res.submessages.is_empty());
    assert!(res.messages.is_empty());
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "psi_distributor_initialized"),
            attr("psi_distributor_addr", psi_distributor_contract.clone()),
        ]
    );

    let config = load_config(deps.as_ref().storage).unwrap();
    assert_eq!(
        config,
        Config {
            nasset_token: Addr::unchecked(nluna_contract_addr.clone()),
            basset_token: Addr::unchecked(msg.basset_token_addr),
            anchor_custody_basset_contract: Addr::unchecked(msg.anchor_custody_basset_contract),
            governance_contract: Addr::unchecked(msg.governance_addr),
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
