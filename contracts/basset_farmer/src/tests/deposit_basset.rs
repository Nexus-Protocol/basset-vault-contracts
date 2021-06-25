use crate::{
    contract::{
        SUBMSG_ID_INIT_NASSET, SUBMSG_ID_INIT_NASSET_STAKER, SUBMSG_ID_INIT_PSI_DISTRIBUTOR,
    },
    response::MsgInstantiateContractResponse,
};

use crate::tests::mock_dependencies;
use cosmwasm_bignumber::Uint256;
use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    to_binary, ContractResult, CosmosMsg, Decimal, Reply, SubcallResponse, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use protobuf::Message;
use yield_optimizer::{
    basset_farmer::{AnyoneMsg, Cw20HookMsg, ExecuteMsg},
    querier::AnchorOverseerMsg,
};

#[test]
fn deposit_basset() {
    let nasset_contract_addr = "addr0001".to_string();
    let basset_token_addr = "addr0002".to_string();
    let custody_basset_contract = "addr0003".to_string();
    let anchor_overseer_contract = "addr0004".to_string();
    let governance_addr = "addr0005".to_string();
    let token_code_id = 10u64; //cw20 contract code
    let nasset_staker_code_id = 11u64; //contract code
    let psi_distributor_code_id = 12u64; //contract code
    let anchor_token = "addr0006".to_string();
    let anchor_market_contract = "addr0007".to_string();
    let anchor_ust_swap_contract = "addr0008".to_string();
    let ust_psi_swap_contract = "addr0009".to_string();
    let aterra_token = "addr0010".to_string();
    let psi_token = "addr0011".to_string();
    let basset_farmer_config_contract = "addr0012".to_string();
    let stable_denom = "addr0013".to_string();
    let nasset_staking_contract = "addr0014".to_string();
    let over_loan_balance_value = "1.01".to_string();
    let psi_distributor_contract = "addr0015".to_string();

    let mut deps = mock_dependencies(&[]);

    //basset_farmer have zero 'nluna' coins
    deps.querier.with_token_balances(&[(
        &nasset_contract_addr,
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128(0))],
    )]);

    // -= INITIALIZATION =-
    {
        let init_msg = yield_optimizer::basset_farmer::InstantiateMsg {
            nasset_token_code_id: token_code_id,
            nasset_staker_code_id,
            psi_distributor_code_id,
            collateral_token_symbol: "Luna".to_string(),
            basset_token_addr: basset_token_addr.clone(),
            anchor_custody_basset_contract: custody_basset_contract.clone(),
            governance_addr: governance_addr.clone(),
            anchor_overseer_contract: anchor_overseer_contract.clone(),
            anchor_token,
            anchor_market_contract,
            anc_stable_swap_contract: anchor_ust_swap_contract,
            psi_stable_swap_contract: ust_psi_swap_contract,
            aterra_token,
            psi_token,
            basset_farmer_config_contract,
            stable_denom,
            claiming_rewards_delay: 1000,
            over_loan_balance_value: over_loan_balance_value.clone(),
        };

        let info = mock_info("addr0000", &[]);
        let _res = crate::contract::instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();
        let mut cw20_instantiate_response = MsgInstantiateContractResponse::new();
        cw20_instantiate_response.set_contract_address(nasset_contract_addr.clone());

        // store nLuna token address
        let reply_msg = Reply {
            id: SUBMSG_ID_INIT_NASSET,
            result: ContractResult::Ok(SubcallResponse {
                events: vec![],
                data: Some(cw20_instantiate_response.write_to_bytes().unwrap().into()),
            }),
        };

        let _res = crate::contract::reply(deps.as_mut(), mock_env(), reply_msg.clone()).unwrap();

        let mut cw20_instantiate_response_2 = MsgInstantiateContractResponse::new();
        cw20_instantiate_response_2.set_contract_address(nasset_staking_contract.clone());
        // store nasset_staker contract address
        let reply_msg_2 = Reply {
            id: SUBMSG_ID_INIT_NASSET_STAKER,
            result: ContractResult::Ok(SubcallResponse {
                events: vec![],
                data: Some(cw20_instantiate_response_2.write_to_bytes().unwrap().into()),
            }),
        };
        let _res = crate::contract::reply(deps.as_mut(), mock_env(), reply_msg_2.clone()).unwrap();

        let mut cw20_instantiate_response_2 = MsgInstantiateContractResponse::new();
        cw20_instantiate_response_2.set_contract_address(psi_distributor_contract.clone());
        // store psi_distributor contract address
        let reply_msg_2 = Reply {
            id: SUBMSG_ID_INIT_PSI_DISTRIBUTOR,
            result: ContractResult::Ok(SubcallResponse {
                events: vec![],
                data: Some(cw20_instantiate_response_2.write_to_bytes().unwrap().into()),
            }),
        };
        let _res = crate::contract::reply(deps.as_mut(), mock_env(), reply_msg_2.clone()).unwrap();
    }

    //first farmer come
    let user_1_address = "addr9999".to_string();
    let deposit_1_amount: Uint128 = 2_000_000_000u128.into();
    {
        // -= USER SEND bAsset tokens to basset_farmer =-
        {
            deps.querier.with_token_balances(&[
                //TODO: why I need that at all?
                // (
                //     &nasset_contract_addr,
                //     &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128(0))],
                // ),
                (
                    &basset_token_addr,
                    &[(&MOCK_CONTRACT_ADDR.to_string(), &deposit_1_amount)],
                ),
            ]);

            let cw20_deposit_msg = Cw20ReceiveMsg {
                sender: user_1_address.clone(),
                amount: deposit_1_amount,
                msg: to_binary(&Cw20HookMsg::Deposit).unwrap(),
            };

            let info = mock_info(&basset_token_addr, &vec![]);
            let res = crate::contract::execute(
                deps.as_mut(),
                mock_env(),
                info,
                ExecuteMsg::Receive(cw20_deposit_msg),
            )
            .unwrap();

            assert_eq!(
                res.messages,
                vec![
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: anchor_overseer_contract.clone(),
                        msg: to_binary(&AnchorOverseerMsg::LockCollateral {
                            collaterals: vec![(
                                basset_token_addr.to_string(),
                                Uint256::from(deposit_1_amount)
                            )],
                        })
                        .unwrap(),
                        send: vec![],
                    }),
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: nasset_contract_addr.clone(),
                        msg: to_binary(&Cw20ExecuteMsg::Mint {
                            recipient: user_1_address.clone(),
                            amount: deposit_1_amount, //first depositer have same amount
                        })
                        .unwrap(),
                        send: vec![],
                    }),
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: MOCK_CONTRACT_ADDR.to_string(),
                        msg: to_binary(&ExecuteMsg::Anyone {
                            anyone_msg: AnyoneMsg::Rebalance,
                        })
                        .unwrap(),
                        send: vec![],
                    }),
                ]
            );
        }
    }

    //second farmer come
    let user_2_address = "addr6666".to_string();
    let deposit_2_amount: Uint128 = 6_000_000_000u128.into();
    {
        deps.querier.with_token_balances(&[
            // TODO: why?
            // (
            //     &nasset_contract_addr,
            //     &[(&MOCK_CONTRACT_ADDR.to_string(), &deposit_1_amount)],
            // ),
            (
                &basset_token_addr,
                &[(
                    &MOCK_CONTRACT_ADDR.to_string(),
                    &(deposit_2_amount + deposit_1_amount),
                )],
            ),
        ]);
        // -= USER SEND bAsset tokens to basset_farmer =-
        {
            let cw20_deposit_msg = Cw20ReceiveMsg {
                sender: user_2_address.clone(),
                amount: deposit_2_amount,
                msg: to_binary(&Cw20HookMsg::Deposit).unwrap(),
            };

            let info = mock_info(&basset_token_addr, &vec![]);
            let res = crate::contract::execute(
                deps.as_mut(),
                mock_env(),
                info,
                ExecuteMsg::Receive(cw20_deposit_msg),
            )
            .unwrap();

            assert_eq!(
                res.messages,
                vec![
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: anchor_overseer_contract.clone(),
                        msg: to_binary(&AnchorOverseerMsg::LockCollateral {
                            collaterals: vec![(
                                basset_token_addr.to_string(),
                                Uint256::from(deposit_2_amount)
                            )],
                        })
                        .unwrap(),
                        send: vec![],
                    }),
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: nasset_contract_addr.clone(),
                        msg: to_binary(&Cw20ExecuteMsg::Mint {
                            recipient: user_2_address.clone(),
                            amount: Uint128::from(6_000_000_000u64), //2B * (6B/8B) / (1 - (6B/8B)) = 6B
                        })
                        .unwrap(),
                        send: vec![],
                    }),
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: MOCK_CONTRACT_ADDR.to_string(),
                        msg: to_binary(&ExecuteMsg::Anyone {
                            anyone_msg: AnyoneMsg::Rebalance,
                        })
                        .unwrap(),
                        send: vec![],
                    }),
                ]
            );
        }
    }
}
