use crate::error::ContractError;
use crate::{
    contract::{execute, instantiate, reply, SUBMSG_ID_INIT_CASSET, SUBMSG_ID_INIT_CASSET_STAKER},
    response::MsgInstantiateContractResponse,
};

use crate::tests::mock_dependencies;
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    attr, to_binary, Addr, BankMsg, Coin, ContractResult, CosmosMsg, Decimal, Reply, ReplyOn,
    Response, StdError, SubMsg, SubcallResponse, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};
use cw20_base::msg::InstantiateMsg as TokenInstantiateMsg;
use protobuf::Message;
use yield_optimizer::casset_staking::{
    AnyoneMsg as CAssetStakingAnyoneMsg, ExecuteMsg as CAssetStakingMsg,
};
use yield_optimizer::{
    basset_farmer::{AnyoneMsg, Cw20HookMsg, ExecuteMsg},
    querier::AnchorOverseerMsg,
};

use super::math::decimal_subtraction;

#[test]
fn deposit_basset() {
    let casset_contract_addr = "addr0001".to_string();
    let basset_token_addr = "addr0002".to_string();
    let custody_basset_contract = "addr0003".to_string();
    let anchor_overseer_contract = "addr0004".to_string();
    let governance_addr = "addr0005".to_string();
    let token_code_id = 10u64; //cw20 contract code
    let casset_staking_code_id = 10u64; //contract code
    let anchor_token = "addr0006".to_string();
    let anchor_market_contract = "addr0007".to_string();
    let anchor_ust_swap_contract = "addr0008".to_string();
    let ust_psi_swap_contract = "addr0009".to_string();
    let aterra_token = "addr0010".to_string();
    let psi_part_in_rewards = Decimal::from_ratio(1u64, 100u64);
    let psi_token = "addr0011".to_string();
    let basset_farmer_config_contract = "addr0012".to_string();
    let stable_denom = "addr0013".to_string();
    let casset_staking_contract = "addr0014".to_string();
    let mut deps = mock_dependencies(&[]);

    //basset_farmer and custody_bluna have zero 'cluna' coins
    deps.querier.with_token_balances(&[(
        &casset_contract_addr,
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128(0))],
    )]);

    // -= INITIALIZATION =-
    {
        let init_msg = yield_optimizer::basset_farmer::InstantiateMsg {
            token_code_id,
            collateral_token_symbol: "Luna".to_string(),
            basset_token_addr: basset_token_addr.clone(),
            custody_basset_contract: custody_basset_contract.clone(),
            governance_addr: governance_addr.clone(),
            anchor_overseer_contract: anchor_overseer_contract.clone(),
            anchor_token,
            anchor_market_contract,
            anchor_ust_swap_contract,
            ust_psi_swap_contract,
            aterra_token,
            psi_part_in_rewards,
            psi_token,
            basset_farmer_config_contract,
            stable_denom,
            casset_staking_code_id,
        };

        let info = mock_info("addr0000", &[]);
        let _res = crate::contract::instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();
        let mut cw20_instantiate_response = MsgInstantiateContractResponse::new();
        cw20_instantiate_response.set_contract_address(casset_contract_addr.clone());

        // store cLuna token address
        let reply_msg = Reply {
            id: SUBMSG_ID_INIT_CASSET,
            result: ContractResult::Ok(SubcallResponse {
                events: vec![],
                data: Some(cw20_instantiate_response.write_to_bytes().unwrap().into()),
            }),
        };

        let _res = crate::contract::reply(deps.as_mut(), mock_env(), reply_msg.clone()).unwrap();

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
        let _res = crate::contract::reply(deps.as_mut(), mock_env(), reply_msg_2.clone()).unwrap();
    }

    //first farmer come
    let user_1_address = "addr9999".to_string();
    let deposit_1_amount: Uint128 = 2_000_000_000u128.into();
    {
        // -= USER SEND bAsset tokens to basset_farmer =-
        {
            deps.querier.with_token_balances(&[
                (
                    &casset_contract_addr,
                    &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128(0))],
                ),
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
                        contract_addr: casset_contract_addr.clone(),
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
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: casset_staking_contract.clone(),
                        msg: to_binary(&CAssetStakingMsg::Anyone {
                            anyone_msg: CAssetStakingAnyoneMsg::UpdateIndex,
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
            (
                &casset_contract_addr,
                &[(&MOCK_CONTRACT_ADDR.to_string(), &deposit_1_amount)],
            ),
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
                        contract_addr: casset_contract_addr.clone(),
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
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: casset_staking_contract.clone(),
                        msg: to_binary(&CAssetStakingMsg::Anyone {
                            anyone_msg: CAssetStakingAnyoneMsg::UpdateIndex,
                        })
                        .unwrap(),
                        send: vec![],
                    }),
                ]
            );
        }
    }
}
