use crate::{
    contract::{SUBMSG_ID_INIT_CASSET, SUBMSG_ID_INIT_CASSET_STAKER},
    response::MsgInstantiateContractResponse,
    state::{load_config, Config},
};

use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{testing::mock_dependencies, Decimal};
use cosmwasm_std::{to_binary, ContractResult, Reply, ReplyOn, SubMsg, SubcallResponse, WasmMsg};
use cw20::MinterResponse;
use cw20_base::msg::InstantiateMsg as TokenInstantiateMsg;
use protobuf::Message;
use yield_optimizer::casset_staking::InstantiateMsg as CAssetStakerInstantiateMsg;

#[test]
fn proper_initialization() {
    let cluna_contract_addr = "addr0001".to_string();
    let token_code_id = 10u64; //cw20 contract code
    let casset_staking_code_id = 11u64; //contract code
    let aterra_token = "addr0010".to_string();
    let stable_denom = "uust".to_string();
    let anchor_market_contract = "addr0007".to_string();
    let casset_staking_contract = "addr0014".to_string();
    let mut deps = mock_dependencies(&[]);

    let msg = yield_optimizer::basset_farmer::InstantiateMsg {
        token_code_id,
        collateral_token_symbol: "Luna".to_string(),
        basset_token_addr: "addr0002".to_string(),
        custody_basset_contract: "addr0003".to_string(),
        anchor_overseer_contract: "addr0004".to_string(),
        governance_addr: "addr0005".to_string(),
        anchor_token: "addr0006".to_string(),
        anchor_market_contract: anchor_market_contract.clone(),
        anc_stable_swap_contract: "addr0008".to_string(),
        psi_stable_swap_contract: "addr0009".to_string(),
        aterra_token: aterra_token.clone(),
        psi_part_in_rewards: Decimal::from_ratio(1u64, 100u64),
        psi_token: "addr0011".to_string(),
        basset_farmer_config_contract: "addr0012".to_string(),
        stable_denom: stable_denom.clone(),
        casset_staking_code_id,
        claiming_rewards_delay: 1000,
    };

    let env = mock_env();
    let info = mock_info("addr0000", &[]);
    let res = crate::contract::instantiate(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        res.submessages,
        vec![SubMsg {
            msg: WasmMsg::Instantiate {
                code_id: token_code_id,
                msg: to_binary(&TokenInstantiateMsg {
                    name: "nexus basset token share representation".to_string(),
                    symbol: "cLuna".to_string(),
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
            id: SUBMSG_ID_INIT_CASSET,
            reply_on: ReplyOn::Success,
        }]
    );

    let mut cw20_instantiate_response = MsgInstantiateContractResponse::new();
    cw20_instantiate_response.set_contract_address(cluna_contract_addr.clone());

    // store cLuna token address
    let reply_msg = Reply {
        id: SUBMSG_ID_INIT_CASSET,
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
                code_id: casset_staking_code_id,
                msg: to_binary(&CAssetStakerInstantiateMsg {
                    casset_token: cluna_contract_addr.clone(),
                    aterra_token: aterra_token.clone(),
                    stable_denom: stable_denom.clone(),
                    basset_farmer_contract: MOCK_CONTRACT_ADDR.to_string(),
                    anchor_market_contract: anchor_market_contract.clone(),
                })
                .unwrap(),
                send: vec![],
                label: "".to_string(),
                admin: None,
            }
            .into(),
            gas_limit: None,
            id: SUBMSG_ID_INIT_CASSET_STAKER,
            reply_on: ReplyOn::Success,
        }]
    );

    // it worked, let's query the state
    let farmer_config: Config = load_config(&deps.storage).unwrap();
    assert_eq!(cluna_contract_addr, farmer_config.casset_token.to_string());

    let mut cw20_instantiate_response_2 = MsgInstantiateContractResponse::new();
    cw20_instantiate_response_2.set_contract_address(casset_staking_contract.clone());
    // store casset_staker contract address
    let reply_msg_2 = Reply {
        id: SUBMSG_ID_INIT_CASSET_STAKER,
        result: ContractResult::Ok(SubcallResponse {
            events: vec![],
            data: Some(cw20_instantiate_response_2.write_to_bytes().unwrap().into()),
        }),
    };
    let res = crate::contract::reply(deps.as_mut(), mock_env(), reply_msg_2.clone()).unwrap();
    assert!(res.submessages.is_empty());
    assert!(res.messages.is_empty());
}
