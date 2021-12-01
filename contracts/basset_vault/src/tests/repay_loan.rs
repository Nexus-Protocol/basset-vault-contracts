use crate::{state::load_repaying_loan_state, SubmsgIds};

use super::sdk::Sdk;
use crate::tests::sdk::{ANCHOR_MARKET_CONTRACT, ATERRA_TOKEN, STABLE_DENOM};
use cosmwasm_bignumber::{Decimal256, Uint256};

use basset_vault::{
    basset_vault_strategy::BorrowerActionResponse,
    querier::{AnchorMarketCw20Msg, AnchorMarketMsg},
    BASSET_VAULT_LOAN_REPAYMENT_MAX_RECURSION_DEEP,
};
use cosmwasm_std::{to_binary, Coin, ReplyOn, Response, SubMsg, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;
use std::str::FromStr;

#[test]
fn repay_loan_without_problems() {
    let mut sdk = Sdk::init();

    let stable_coin_balance = Uint128::from(200u64);
    let loan_to_repay = Uint256::from(10_000u64);
    let advised_buffer_size = Uint256::from(50u64);
    let aterra_balance = Uint256::from(200u64);
    let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
    let tax_rate = Decimal256::from_str("0.2").unwrap();

    sdk.set_stable_balance(stable_coin_balance);
    sdk.set_aterra_balance(aterra_balance);
    sdk.set_borrower_action(BorrowerActionResponse::Repay {
        amount: loan_to_repay,
        advised_buffer_size,
    });
    sdk.set_aterra_exchange_rate(aterra_exchange_rate);
    sdk.set_tax(tax_rate.into(), 10u128);

    println!("xxxxxxxx");
    // -= REBALANCE =-
    {
        let response = sdk.rebalance().unwrap();
        assert_eq!(
            response.messages,
            vec![SubMsg {
                msg: WasmMsg::Execute {
                    contract_addr: ATERRA_TOKEN.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Send {
                        contract: ANCHOR_MARKET_CONTRACT.to_string(),
                        amount: aterra_balance.into(),
                        msg: to_binary(&AnchorMarketCw20Msg::RedeemStable {}).unwrap(),
                    })
                    .unwrap(),
                    funds: vec![],
                }
                .into(),
                gas_limit: None,
                id: SubmsgIds::RedeemStableOnRepayLoan.id(),
                reply_on: ReplyOn::Always,
            }]
        );
        let rapaying_state = load_repaying_loan_state(&sdk.deps.storage).unwrap();
        assert_eq!(rapaying_state.iteration_index, 0);
        assert_eq!(rapaying_state.to_repay_amount, loan_to_repay);
        assert_eq!(rapaying_state.repaying_amount, Uint256::zero());
        assert_eq!(rapaying_state.aim_buffer_size, advised_buffer_size);
    }

    println!("===============");
    println!("===============");
    println!("===============");
    println!("===============");

    // -= ANCHOR REDEEM SUCCESSFULL =-
    {
        //anchor send stables for selling aterra
        // 10 is a cap tax
        sdk.set_stable_balance(
            (Uint256::from(stable_coin_balance) + aterra_balance * aterra_exchange_rate
                - Uint256::from(10u64))
            .into(),
        );
        //all aterra was redeemed
        sdk.set_aterra_balance(Uint256::zero());

        let response = sdk.aterra_redeem_success().unwrap();

        let repay_stable_coin = Coin {
            denom: STABLE_DENOM.to_string(),
            // 10 is a cap tax to repay loan
            // 10 is a cap tax that anchor pay to send stables to us
            amount: (Uint256::from(stable_coin_balance) + aterra_balance * aterra_exchange_rate
                - Uint256::from(10u64)
                - Uint256::from(10u64))
            .into(),
        };
        assert_eq!(
            response.messages,
            vec![SubMsg {
                msg: WasmMsg::Execute {
                    contract_addr: ANCHOR_MARKET_CONTRACT.to_string(),
                    msg: to_binary(&AnchorMarketMsg::RepayStable {}).unwrap(),
                    funds: vec![repay_stable_coin.clone()],
                }
                .into(),
                gas_limit: None,
                id: SubmsgIds::RepayLoan.id(),
                reply_on: ReplyOn::Success,
            }]
        );

        let rapaying_state = load_repaying_loan_state(&sdk.deps.storage).unwrap();
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
    let mut sdk = Sdk::init();

    let stable_coin_initial_balance = Uint128::new(5_000);
    sdk.set_stable_balance(stable_coin_initial_balance);
    sdk.set_aterra_exchange_rate(Decimal256::from_str("1.25").unwrap());
    let aterra_balance = Uint256::from(7_000u64);
    sdk.set_aterra_balance(aterra_balance);

    //no tax
    sdk.set_tax(Decimal256::zero().into(), 99999999999u128);

    // -= asking for REPAY =-
    {
        let to_repay_amount = Uint256::from(10_000u64);
        let aim_buffer_size = Uint256::from(5_000u64);
        sdk.set_borrower_action(BorrowerActionResponse::Repay {
            amount: to_repay_amount,
            advised_buffer_size: aim_buffer_size,
        });
        let response = sdk.rebalance().unwrap();

        assert_eq!(
            response.messages,
            vec![SubMsg {
                msg: WasmMsg::Execute {
                    contract_addr: ATERRA_TOKEN.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Send {
                        contract: ANCHOR_MARKET_CONTRACT.to_string(),
                        amount: aterra_balance.into(),
                        msg: to_binary(&AnchorMarketCw20Msg::RedeemStable {}).unwrap(),
                    })
                    .unwrap(),
                    funds: vec![],
                }
                .into(),
                gas_limit: None,
                id: SubmsgIds::RedeemStableOnRepayLoan.id(),
                reply_on: ReplyOn::Always,
            }]
        );
    }

    // -= REDEEM failed =-
    {
        let response = sdk.aterra_redeed_failed().unwrap();
        //now contract should repay loan with buffer and try to redeem aterra for that amount
        assert_eq!(
            response.messages,
            vec![
                SubMsg {
                    msg: WasmMsg::Execute {
                        contract_addr: ANCHOR_MARKET_CONTRACT.to_string(),
                        msg: to_binary(&AnchorMarketMsg::RepayStable {}).unwrap(),
                        funds: vec![Coin {
                            denom: STABLE_DENOM.to_string(),
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
                        contract_addr: ATERRA_TOKEN.to_string(),
                        msg: to_binary(&Cw20ExecuteMsg::Send {
                            contract: ANCHOR_MARKET_CONTRACT.to_string(),
                            //sell aterra for same value as repaying long (4000*1.25 = 5k)
                            amount: Uint128::from(4_000u64),
                            msg: to_binary(&AnchorMarketCw20Msg::RedeemStable {}).unwrap(),
                        })
                        .unwrap(),
                        funds: vec![],
                    }
                    .into(),
                    gas_limit: None,
                    id: SubmsgIds::RedeemStableOnRepayLoan.id(),
                    reply_on: ReplyOn::Success,
                }
            ]
        );
    }

    // -= REDEEM SUCCEEDED =-
    {
        let updated_aterra_balance = aterra_balance - Uint256::from(4_000u64);
        sdk.set_aterra_balance(updated_aterra_balance);
        sdk.continue_repay_loan().unwrap();

        let updated_repaying_state = load_repaying_loan_state(sdk.deps.as_mut().storage).unwrap();
        assert_eq!(
            updated_repaying_state.to_repay_amount,
            Uint256::from(5_000u64)
        );

        let response = sdk.aterra_redeem_success().unwrap();
        assert_eq!(
            response.messages,
            vec![
                SubMsg {
                    msg: WasmMsg::Execute {
                        contract_addr: ANCHOR_MARKET_CONTRACT.to_string(),
                        msg: to_binary(&AnchorMarketMsg::RepayStable {}).unwrap(),
                        funds: vec![Coin {
                            denom: STABLE_DENOM.to_string(),
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
                        contract_addr: ATERRA_TOKEN.to_string(),
                        msg: to_binary(&Cw20ExecuteMsg::Send {
                            contract: ANCHOR_MARKET_CONTRACT.to_string(),
                            //sell rest of aterra 3k (3000*1.25 = 3750)
                            amount: Uint128::from(3_000u64),
                            msg: to_binary(&AnchorMarketCw20Msg::RedeemStable {}).unwrap(),
                        })
                        .unwrap(),
                        funds: vec![],
                    }
                    .into(),
                    gas_limit: None,
                    id: SubmsgIds::RedeemStableOnRepayLoan.id(),
                    reply_on: ReplyOn::Success,
                }
            ]
        );
    }

    // -= third step =-
    {
        sdk.set_aterra_balance(Uint256::zero());
        sdk.continue_repay_loan().unwrap();
        let updated_repaying_state = load_repaying_loan_state(sdk.deps.as_mut().storage).unwrap();
        assert_eq!(updated_repaying_state.to_repay_amount, Uint256::zero());

        let response = sdk.aterra_redeem_success().unwrap();
        assert_eq!(response, Response::default());
    }
}

#[test]
fn limited_recursion_depth_all_errors() {
    let mut sdk = Sdk::init();

    let stable_coin_initial_balance = Uint128::new(5_000);
    sdk.set_stable_balance(stable_coin_initial_balance);
    sdk.set_aterra_exchange_rate(Decimal256::from_str("1.25").unwrap());
    let aterra_balance = Uint256::from(7_000u64);
    sdk.set_aterra_balance(aterra_balance);

    //no tax
    sdk.set_tax(Decimal256::zero().into(), 99999999999u128);

    let to_repay_amount = Uint256::from(10_000u64);
    let aim_buffer_size = Uint256::from(5_000u64);
    sdk.set_borrower_action(BorrowerActionResponse::Repay {
        amount: to_repay_amount,
        advised_buffer_size: aim_buffer_size,
    });

    // -= REPAY =-
    sdk.rebalance().unwrap();

    // -= ANCHOR REDEEM FAILED =-
    let start_from = 1;
    for repaying_index in start_from..BASSET_VAULT_LOAN_REPAYMENT_MAX_RECURSION_DEEP {
        let response = sdk.aterra_redeed_failed().unwrap();
        //now contract should repay loan with buffer and try to redeem aterra for that amount
        assert_eq!(
            response.messages,
            vec![
                SubMsg {
                    msg: WasmMsg::Execute {
                        contract_addr: ANCHOR_MARKET_CONTRACT.to_string(),
                        msg: to_binary(&AnchorMarketMsg::RepayStable {}).unwrap(),
                        funds: vec![Coin {
                            denom: STABLE_DENOM.to_string(),
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
                        contract_addr: ATERRA_TOKEN.to_string(),
                        msg: to_binary(&Cw20ExecuteMsg::Send {
                            contract: ANCHOR_MARKET_CONTRACT.to_string(),
                            //sell aterra for same value as repaying long (4000*1.25 = 5k)
                            amount: Uint128::from(4_000u64),
                            msg: to_binary(&AnchorMarketCw20Msg::RedeemStable {}).unwrap(),
                        })
                        .unwrap(),
                        funds: vec![],
                    }
                    .into(),
                    gas_limit: None,
                    id: SubmsgIds::RedeemStableOnRepayLoan.id(),
                    reply_on: ReplyOn::Success,
                }
            ]
        );
        let rapaying_state = load_repaying_loan_state(&sdk.deps.storage).unwrap();
        assert_eq!(rapaying_state.iteration_index, repaying_index);
    }

    let response = sdk.aterra_redeed_failed();
    assert!(response.is_err());
}

#[test]
fn limited_recursion_depth_repayed_something() {
    let mut sdk = Sdk::init();

    let stable_coin_initial_balance = Uint128::new(5_000);
    sdk.set_stable_balance(stable_coin_initial_balance);
    sdk.set_aterra_exchange_rate(Decimal256::from_str("1.25").unwrap());
    let aterra_balance = Uint256::from(7_000u64);
    sdk.set_aterra_balance(aterra_balance);

    //no tax
    sdk.set_tax(Decimal256::zero().into(), 99999999999u128);

    let to_repay_amount = Uint256::from(10_000u64);
    let aim_buffer_size = Uint256::from(5_000u64);
    sdk.set_borrower_action(BorrowerActionResponse::Repay {
        amount: to_repay_amount,
        advised_buffer_size: aim_buffer_size,
    });

    // -= REPAY =-
    sdk.rebalance().unwrap();

    // -= ANCHOR REDEEM FAILED =-
    let start_from = 2;
    for repaying_index in start_from..BASSET_VAULT_LOAN_REPAYMENT_MAX_RECURSION_DEEP {
        let response = sdk.aterra_redeed_failed().unwrap();
        //now contract should repay loan with buffer and try to redeem aterra for that amount
        assert_eq!(
            response.messages,
            vec![
                SubMsg {
                    msg: WasmMsg::Execute {
                        contract_addr: ANCHOR_MARKET_CONTRACT.to_string(),
                        msg: to_binary(&AnchorMarketMsg::RepayStable {}).unwrap(),
                        funds: vec![Coin {
                            denom: STABLE_DENOM.to_string(),
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
                        contract_addr: ATERRA_TOKEN.to_string(),
                        msg: to_binary(&Cw20ExecuteMsg::Send {
                            contract: ANCHOR_MARKET_CONTRACT.to_string(),
                            //sell aterra for same value as repaying long (4000*1.25 = 5k)
                            amount: Uint128::from(4_000u64),
                            msg: to_binary(&AnchorMarketCw20Msg::RedeemStable {}).unwrap(),
                        })
                        .unwrap(),
                        funds: vec![],
                    }
                    .into(),
                    gas_limit: None,
                    id: SubmsgIds::RedeemStableOnRepayLoan.id(),
                    reply_on: ReplyOn::Success,
                }
            ]
        );
        let rapaying_state = load_repaying_loan_state(&sdk.deps.storage).unwrap();
        assert_eq!(rapaying_state.iteration_index, repaying_index - 1);
    }
    //one repaying is OK!
    sdk.continue_repay_loan().unwrap();

    let response = sdk.aterra_redeed_failed();
    assert!(response.is_ok());
}

#[test]
fn reset_iteration_index() {
    let mut sdk = Sdk::init();

    let stable_coin_initial_balance = Uint128::new(5_000);
    sdk.set_stable_balance(stable_coin_initial_balance);
    sdk.set_aterra_exchange_rate(Decimal256::from_str("1.25").unwrap());
    let aterra_balance = Uint256::from(7_000u64);
    sdk.set_aterra_balance(aterra_balance);

    //no tax
    sdk.set_tax(Decimal256::zero().into(), 99999999999u128);

    let to_repay_amount = Uint256::from(10_000u64);
    let aim_buffer_size = Uint256::from(5_000u64);
    sdk.set_borrower_action(BorrowerActionResponse::Repay {
        amount: to_repay_amount,
        advised_buffer_size: aim_buffer_size,
    });

    // -= REPAY =-
    sdk.rebalance().unwrap();

    // -= ANCHOR REDEEM FAILED =-
    let start_from = 1;
    for _ in start_from..BASSET_VAULT_LOAN_REPAYMENT_MAX_RECURSION_DEEP {
        let response = sdk.aterra_redeed_failed();
        assert!(response.is_ok());
    }

    let response = sdk.aterra_redeed_failed();
    assert!(response.is_err());
    let rapaying_state = load_repaying_loan_state(&sdk.deps.storage).unwrap();
    assert_eq!(
        rapaying_state.iteration_index,
        BASSET_VAULT_LOAN_REPAYMENT_MAX_RECURSION_DEEP - 1
    );

    // -= SECOND REPAY =-
    sdk.rebalance().unwrap();
    let rapaying_state = load_repaying_loan_state(&sdk.deps.storage).unwrap();
    assert_eq!(rapaying_state.iteration_index, 0);
}
