use crate::{queries, response::MsgInstantiateContractResponse, state::Config};

use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    to_binary, Api, ContractResult, Reply, ReplyOn, SubMsg, SubcallResponse, WasmMsg,
};
use cw20::MinterResponse;
use cw20_base::msg::InstantiateMsg as TokenInstantiateMsg;
use protobuf::Message;

#[test]
fn proper_initialization() {
    let cluna_contract_addr = "addr0001".to_string();
    let token_code_id = 10u64; //cw20 contract code
    let mut deps = mock_dependencies(&[]);

    let msg = yield_optimizer::basset_farmer::InstantiateMsg {
        token_code_id,
        collateral_token_symbol: "Luna".to_string(),
        basset_token_addr: "addr0002".to_string(),
        custody_basset_contract: "addr0003".to_string(),
        overseer_addr: "addr0004".to_string(),
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
            id: 1,
            reply_on: ReplyOn::Success,
        }]
    );

    let mut cw20_instantiate_response = MsgInstantiateContractResponse::new();
    cw20_instantiate_response.set_contract_address(cluna_contract_addr.clone());

    // store cLuna token address
    let reply_msg = Reply {
        id: 1,
        result: ContractResult::Ok(SubcallResponse {
            events: vec![],
            data: Some(cw20_instantiate_response.write_to_bytes().unwrap().into()),
        }),
    };

    let _res = crate::contract::reply(deps.as_mut(), mock_env(), reply_msg.clone()).unwrap();

    // it worked, let's query the state
    let farmer_config: Config = queries::query_config(deps.as_ref()).unwrap();
    let casset_token_addr = deps.api.addr_humanize(&farmer_config.casset_token).unwrap();
    assert_eq!(cluna_contract_addr, casset_token_addr.to_string());
}
