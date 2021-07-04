use super::instantiate::instantiate_basset_farmer;
use crate::{
    state::{load_repaying_loan_state, store_config, RepayingLoanState},
    SubmsgIds, TOO_HIGH_BORROW_DEMAND_ERR_MSG,
};

use crate::tests::mock_dependencies;
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    to_binary, Addr, Coin, ContractResult, Decimal, Reply, ReplyOn, Response, SubMsg,
    SubcallResponse, Uint128, WasmMsg,
};
use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;
use std::str::FromStr;
use yield_optimizer::{
    basset_farmer::ExecuteMsg,
    basset_farmer_config::BorrowerActionResponse,
    querier::{
        AnchorMarketCw20Msg, AnchorMarketEpochStateResponse, AnchorMarketMsg, BorrowerInfoResponse,
    },
};

#[test]
fn repay_loan_without_problems() {
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
    let basset_token_addr = "addr0002".to_string();
    let anchor_overseer_contract = "addr0004".to_string();
    let anchor_custody_basset_contract = "addr0003".to_string();
    let basset_farmer_config_contract = "addr0011".to_string();

    let msg = yield_optimizer::basset_farmer::InstantiateMsg {
        nasset_token_code_id,
        nasset_token_config_holder_code_id,
        nasset_token_rewards_code_id,
        psi_distributor_code_id,
        collateral_token_symbol: "Luna".to_string(),
        basset_token_addr: basset_token_addr.clone(),
        anchor_custody_basset_contract: anchor_custody_basset_contract.clone(),
        anchor_overseer_contract: anchor_overseer_contract.clone(),
        governance_contract: governance_contract.clone(),
        anchor_token: "addr0006".to_string(),
        anchor_market_contract: anchor_market_contract.clone(),
        anc_stable_swap_contract: "addr0008".to_string(),
        psi_stable_swap_contract: "addr0009".to_string(),
        aterra_token: aterra_token.clone(),
        psi_token: psi_token.clone(),
        basset_farmer_config_contract: basset_farmer_config_contract.clone(),
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

    let stable_coin_balance = Uint128::from(200u64);
    let loan_to_repay = Uint256::from(10_000u64);
    let locked_basset_amount = Uint128::from(10_000u64);
    let basset_farmer_loan_amount = Uint256::from(10_000u64);
    let advised_buffer_size = Uint256::from(50u64);
    let aterra_balance = Uint256::from(200u64);
    let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();

    deps.querier.with_token_balances(&[
        (
            &anchor_custody_basset_contract,
            &[(&MOCK_CONTRACT_ADDR.to_string(), &locked_basset_amount)],
        ),
        (
            &aterra_token,
            &[(&MOCK_CONTRACT_ADDR.to_string(), &aterra_balance.into())],
        ),
    ]);
    deps.querier.with_loan(&[(
        &anchor_market_contract,
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &BorrowerInfoResponse {
                borrower: MOCK_CONTRACT_ADDR.to_string(),
                interest_index: Decimal256::one(),
                reward_index: Decimal256::zero(),
                loan_amount: basset_farmer_loan_amount,
                pending_rewards: Decimal256::zero(),
            },
        )],
    )]);
    deps.querier.with_wasm_query_response(&[
        (
            &basset_farmer_config_contract,
            &to_binary(&BorrowerActionResponse::Repay {
                amount: loan_to_repay,
                advised_buffer_size,
            })
            .unwrap(),
        ),
        (
            &anchor_market_contract,
            &to_binary(&AnchorMarketEpochStateResponse {
                exchange_rate: aterra_exchange_rate,
                aterra_supply: Uint256::from(1_000_000u64),
            })
            .unwrap(),
        ),
    ]);

    deps.querier.with_tax(
        Decimal::percent(20),
        &[(&stable_denom.to_string(), &Uint128::from(10u128))],
    );

    // -= REBALANCE =-
    {
        let rebalance_msg = yield_optimizer::basset_farmer::AnyoneMsg::Rebalance;
        let info = mock_info("addr8888", &vec![]);
        let res = crate::contract::execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Anyone {
                anyone_msg: rebalance_msg,
            },
        )
        .unwrap();
        assert_eq!(
            res.submessages,
            vec![SubMsg {
                msg: WasmMsg::Execute {
                    contract_addr: aterra_token.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Send {
                        contract: anchor_market_contract.to_string(),
                        amount: aterra_balance.into(),
                        msg: to_binary(&AnchorMarketCw20Msg::RedeemStable {}).unwrap(),
                    })
                    .unwrap(),
                    send: vec![],
                }
                .into(),
                gas_limit: None,
                id: SubmsgIds::RedeemStableOnRepayLoan.id(),
                reply_on: ReplyOn::Always,
            }]
        );
        let rapaying_state = load_repaying_loan_state(&deps.storage).unwrap();
        assert_eq!(rapaying_state.iteration_index, 0);
        assert_eq!(rapaying_state.to_repay_amount, loan_to_repay);
        assert_eq!(rapaying_state.repaying_amount, Uint256::zero());
        assert_eq!(rapaying_state.aim_buffer_size, advised_buffer_size);

        //sending Ok reply, means aterra was successfuly redeemed
        let reply_msg = Reply {
            id: SubmsgIds::RedeemStableOnRepayLoan.id(),
            result: ContractResult::Ok(SubcallResponse {
                events: vec![],
                //we don't use it
                data: None,
            }),
        };

        //anchor send stables for selling aterra
        deps.querier.update_base_balance(
            MOCK_CONTRACT_ADDR,
            vec![Coin {
                denom: stable_denom.to_string(),
                // 10 is a cap tax
                amount: (Uint256::from(stable_coin_balance)
                    + aterra_balance * aterra_exchange_rate
                    - Uint256::from(10u64))
                .into(),
            }],
        );
        //all aterra was redeemed
        deps.querier.with_token_balances(&[(
            &aterra_token,
            &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::zero())],
        )]);

        let repay_stable_coin = Coin {
            denom: stable_denom.to_string(),
            // 10 is a cap tax to repay loan
            // 10 is a cap tax that anchor pay to send stables to us
            amount: (Uint256::from(stable_coin_balance) + aterra_balance * aterra_exchange_rate
                - Uint256::from(10u64)
                - Uint256::from(10u64))
            .into(),
        };
        let res = crate::contract::reply(deps.as_mut(), mock_env(), reply_msg.clone()).unwrap();
        assert_eq!(
            res.submessages,
            vec![SubMsg {
                msg: WasmMsg::Execute {
                    contract_addr: anchor_market_contract.to_string(),
                    msg: to_binary(&AnchorMarketMsg::RepayStable {}).unwrap(),
                    send: vec![repay_stable_coin.clone()],
                }
                .into(),
                gas_limit: None,
                id: SubmsgIds::RepayLoan.id(),
                reply_on: ReplyOn::Success,
            }]
        );

        let rapaying_state = load_repaying_loan_state(&deps.storage).unwrap();
        assert_eq!(rapaying_state.iteration_index, 1);
        assert_eq!(rapaying_state.to_repay_amount, loan_to_repay);
        assert_eq!(
            rapaying_state.repaying_amount,
            repay_stable_coin.amount.into()
        );
        assert_eq!(rapaying_state.aim_buffer_size, advised_buffer_size);
    }
}

#[test]
fn repay_loan_fail_to_redeem_aterra() {
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
    let basset_token_addr = "addr0002".to_string();
    let anchor_overseer_contract = "addr0004".to_string();
    let anchor_custody_basset_contract = "addr0003".to_string();
    let basset_farmer_config_contract = "addr0011".to_string();
    let anchor_token = "addr0006".to_string();
    let anc_stable_swap_contract = "addr0008".to_string();
    let psi_stable_swap_contract = "addr0009".to_string();

    let msg = yield_optimizer::basset_farmer::InstantiateMsg {
        nasset_token_code_id,
        nasset_token_config_holder_code_id,
        nasset_token_rewards_code_id,
        psi_distributor_code_id,
        collateral_token_symbol: "Luna".to_string(),
        basset_token_addr: basset_token_addr.clone(),
        anchor_custody_basset_contract: anchor_custody_basset_contract.clone(),
        anchor_overseer_contract: anchor_overseer_contract.clone(),
        governance_contract: governance_contract.clone(),
        anchor_token: anchor_token.clone(),
        anchor_market_contract: anchor_market_contract.clone(),
        anc_stable_swap_contract: anc_stable_swap_contract.clone(),
        psi_stable_swap_contract: psi_stable_swap_contract.clone(),
        aterra_token: aterra_token.clone(),
        psi_token: psi_token.clone(),
        basset_farmer_config_contract: basset_farmer_config_contract.clone(),
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

    let basset_farmer_config = crate::state::Config {
        anchor_custody_basset_contract: Addr::unchecked(anchor_custody_basset_contract.clone()),
        governance_contract: Addr::unchecked(governance_contract.clone()),
        anchor_overseer_contract: Addr::unchecked(anchor_overseer_contract.clone()),
        anchor_token: Addr::unchecked(anchor_token.clone()),
        nasset_token: Addr::unchecked(nasset_contract_addr.clone()),
        basset_token: Addr::unchecked(basset_token_addr.clone()),
        anchor_market_contract: Addr::unchecked(anchor_market_contract.clone()),
        anc_stable_swap_contract: Addr::unchecked(anc_stable_swap_contract.clone()),
        psi_stable_swap_contract: Addr::unchecked(psi_stable_swap_contract.clone()),
        aterra_token: Addr::unchecked(aterra_token.clone()),
        psi_token: Addr::unchecked(psi_token.clone()),
        basset_farmer_config_contract: Addr::unchecked(basset_farmer_config_contract.clone()),
        stable_denom: stable_denom.to_string(),
        claiming_rewards_delay: 1000,
        over_loan_balance_value: Decimal256::from_str(&over_loan_balance_value).unwrap(),
        psi_distributor_addr: Addr::unchecked(psi_distributor_contract),
    };

    let stable_coin_initial_balance = Uint128::from(5_000u64);
    let mut deps = mock_dependencies(&[Coin {
        denom: stable_denom.clone(),
        amount: stable_coin_initial_balance,
    }]);
    deps.querier.with_wasm_query_response(&[(
        &anchor_market_contract,
        &to_binary(&AnchorMarketEpochStateResponse {
            exchange_rate: Decimal256::from_str("1.25").unwrap(),
            aterra_supply: Uint256::from(1_000_000u64),
        })
        .unwrap(),
    )]);
    let aterra_balance = Uint256::from(7_000u64);
    deps.querier.with_token_balances(&[(
        &aterra_token,
        &[(&MOCK_CONTRACT_ADDR.to_string(), &aterra_balance.into())],
    )]);
    //no tax
    deps.querier.with_tax(
        Decimal::zero(),
        &[(&stable_denom.to_string(), &Uint128::from(99999999999u128))],
    );
    store_config(&mut deps.storage, &basset_farmer_config).unwrap();

    // -= asking for REPAY =-
    let to_repay_amount = Uint256::from(10_000u64);
    let aim_buffer_size = Uint256::from(5_000u64);
    let repaying_loan_state = RepayingLoanState {
        iteration_index: 0,
        to_repay_amount,
        repaying_amount: Uint256::zero(),
        aim_buffer_size,
    };

    let repay_response = crate::commands::repay_logic(
        deps.as_mut(),
        mock_env(),
        &basset_farmer_config,
        repaying_loan_state,
    )
    .unwrap();

    assert_eq!(
        repay_response.submessages,
        vec![SubMsg {
            msg: WasmMsg::Execute {
                contract_addr: aterra_token.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Send {
                    contract: anchor_market_contract.to_string(),
                    amount: aterra_balance.into(),
                    msg: to_binary(&AnchorMarketCw20Msg::RedeemStable {}).unwrap(),
                })
                .unwrap(),
                send: vec![],
            }
            .into(),
            gas_limit: None,
            id: SubmsgIds::RedeemStableOnRepayLoan.id(),
            reply_on: ReplyOn::Always,
        }]
    );

    // -= REDEEM failed =-
    let reply_1_msg = Reply {
        id: SubmsgIds::RedeemStableOnRepayLoan.id(),
        result: ContractResult::Err(format!(
            "fail to redeem aterra, cause: {}",
            TOO_HIGH_BORROW_DEMAND_ERR_MSG,
        )),
    };
    let reply_1_response =
        crate::contract::reply(deps.as_mut(), mock_env(), reply_1_msg.clone()).unwrap();
    //now contract should repay loan with buffer and try to redeem aterra for that amount
    assert_eq!(
        reply_1_response.submessages,
        vec![
            SubMsg {
                msg: WasmMsg::Execute {
                    contract_addr: anchor_market_contract.clone(),
                    msg: to_binary(&AnchorMarketMsg::RepayStable {}).unwrap(),
                    send: vec![Coin {
                        denom: stable_denom.clone(),
                        amount: stable_coin_initial_balance,
                    }],
                }
                .into(),
                gas_limit: None,
                id: SubmsgIds::RepayLoan.id(),
                reply_on: ReplyOn::Success,
            },
            SubMsg {
                msg: WasmMsg::Execute {
                    contract_addr: aterra_token.clone(),
                    msg: to_binary(&Cw20ExecuteMsg::Send {
                        contract: anchor_market_contract.clone(),
                        //sell aterra for same value as repaying long (4000*1.25 = 5k)
                        amount: Uint128::from(4_000u64),
                        msg: to_binary(&AnchorMarketCw20Msg::RedeemStable {}).unwrap(),
                    })
                    .unwrap(),
                    send: vec![],
                }
                .into(),
                gas_limit: None,
                id: SubmsgIds::RedeemStableOnRepayLoan.id(),
                reply_on: ReplyOn::Success,
            }
        ]
    );

    // -= REDEEM SUCCEEDED =-
    let updated_aterra_balance = aterra_balance - Uint256::from(4_000u64);
    deps.querier.with_token_balances(&[(
        &aterra_token,
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &updated_aterra_balance.into(),
        )],
    )]);
    let reply_2_msg = Reply {
        id: SubmsgIds::RepayLoan.id(),
        result: ContractResult::Ok(SubcallResponse {
            events: vec![],
            data: None,
        }),
    };
    let _reply_2_response =
        crate::contract::reply(deps.as_mut(), mock_env(), reply_2_msg.clone()).unwrap();
    let updated_repaying_state = load_repaying_loan_state(deps.as_mut().storage).unwrap();
    assert_eq!(
        updated_repaying_state.to_repay_amount,
        Uint256::from(5_000u64)
    );

    let reply_3_msg = Reply {
        id: SubmsgIds::RedeemStableOnRepayLoan.id(),
        result: ContractResult::Ok(SubcallResponse {
            events: vec![],
            data: None,
        }),
    };
    let reply_3_response =
        crate::contract::reply(deps.as_mut(), mock_env(), reply_3_msg.clone()).unwrap();
    assert_eq!(
        reply_3_response.submessages,
        vec![
            SubMsg {
                msg: WasmMsg::Execute {
                    contract_addr: anchor_market_contract.clone(),
                    msg: to_binary(&AnchorMarketMsg::RepayStable {}).unwrap(),
                    send: vec![Coin {
                        denom: stable_denom.clone(),
                        amount: Uint128::from(5_000u64),
                    }],
                }
                .into(),
                gas_limit: None,
                id: SubmsgIds::RepayLoan.id(),
                reply_on: ReplyOn::Success,
            },
            SubMsg {
                msg: WasmMsg::Execute {
                    contract_addr: aterra_token.clone(),
                    msg: to_binary(&Cw20ExecuteMsg::Send {
                        contract: anchor_market_contract.clone(),
                        //sell rest of aterra 3k (3000*1.25 = 3750)
                        amount: Uint128::from(3_000u64),
                        msg: to_binary(&AnchorMarketCw20Msg::RedeemStable {}).unwrap(),
                    })
                    .unwrap(),
                    send: vec![],
                }
                .into(),
                gas_limit: None,
                id: SubmsgIds::RedeemStableOnRepayLoan.id(),
                reply_on: ReplyOn::Success,
            }
        ]
    );

    // -= third step =-
    deps.querier.with_token_balances(&[(
        &aterra_token,
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::zero())],
    )]);
    let reply_4_msg = Reply {
        id: SubmsgIds::RepayLoan.id(),
        result: ContractResult::Ok(SubcallResponse {
            events: vec![],
            data: None,
        }),
    };
    let _reply_4_response =
        crate::contract::reply(deps.as_mut(), mock_env(), reply_4_msg.clone()).unwrap();
    let updated_repaying_state = load_repaying_loan_state(deps.as_mut().storage).unwrap();
    assert_eq!(updated_repaying_state.to_repay_amount, Uint256::zero());
    let reply_5_msg = Reply {
        id: SubmsgIds::RedeemStableOnRepayLoan.id(),
        result: ContractResult::Ok(SubcallResponse {
            events: vec![],
            data: None,
        }),
    };
    let reply_5_response =
        crate::contract::reply(deps.as_mut(), mock_env(), reply_5_msg.clone()).unwrap();
    assert_eq!(reply_5_response, Response::default());
}
