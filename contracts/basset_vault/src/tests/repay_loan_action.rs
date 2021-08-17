use crate::state::load_config;
use crate::{utils::RepayLoanAction, SubmsgIds};

use super::sdk::Sdk;
use crate::tests::sdk::{ANCHOR_MARKET_CONTRACT, ATERRA_TOKEN, STABLE_DENOM};
use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{to_binary, Coin, Response, SubMsg, WasmMsg};
use cw20::Cw20ExecuteMsg;

use basset_vault::querier::{AnchorMarketCw20Msg, AnchorMarketMsg};

#[test]
fn repay_loan_action_to_response_nothing() {
    let sdk = Sdk::init();
    let config = load_config(&sdk.deps.storage).unwrap();

    let repay_loan_action = RepayLoanAction::Nothing;
    let response = repay_loan_action.to_response(&config).unwrap();
    assert_eq!(response, Response::default());
}

#[test]
fn repay_loan_action_to_response_repay_loan() {
    let sdk = Sdk::init();
    let config = load_config(&sdk.deps.storage).unwrap();

    let repay_amount = Uint256::from(2_000u64);
    let repay_loan_action = RepayLoanAction::RepayLoan {
        amount: repay_amount,
    };
    let response = repay_loan_action.to_response(&config).unwrap();

    let repay_stable_coin = Coin {
        denom: STABLE_DENOM.to_string(),
        amount: repay_amount.into(),
    };
    let expected_response = Response::new()
        .add_submessage(SubMsg::reply_on_success(
            WasmMsg::Execute {
                contract_addr: ANCHOR_MARKET_CONTRACT.to_string(),
                msg: to_binary(&AnchorMarketMsg::RepayStable {}).unwrap(),
                funds: vec![repay_stable_coin],
            },
            SubmsgIds::RepayLoan.id(),
        ))
        .add_attributes(vec![
            ("action", "repay_loan"),
            ("amount", &repay_amount.to_string()),
        ]);
    assert_eq!(response, expected_response);
}

#[test]
fn repay_loan_action_to_response_sell_aterra() {
    let sdk = Sdk::init();
    let config = load_config(&sdk.deps.storage).unwrap();

    let sell_amount = Uint256::from(2_000u64);
    let repay_loan_action = RepayLoanAction::SellAterra {
        amount: sell_amount,
    };
    let response = repay_loan_action.to_response(&config).unwrap();

    let expected_response = Response::new()
        .add_submessage(SubMsg::reply_always(
            WasmMsg::Execute {
                contract_addr: ATERRA_TOKEN.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Send {
                    contract: ANCHOR_MARKET_CONTRACT.to_string(),
                    amount: sell_amount.into(),
                    msg: to_binary(&AnchorMarketCw20Msg::RedeemStable {}).unwrap(),
                })
                .unwrap(),
                funds: vec![],
            },
            SubmsgIds::RedeemStableOnRepayLoan.id(),
        ))
        .add_attributes(vec![
            ("action", "sell_aterra"),
            ("amount", &sell_amount.to_string()),
        ]);
    assert_eq!(response, expected_response);
}

#[test]
fn repay_loan_action_to_response_repay_loan_and_sell_aterra() {
    let sdk = Sdk::init();
    let config = load_config(&sdk.deps.storage).unwrap();

    let repay_loan_amount = Uint256::from(5_000u64);
    let aterra_amount_to_sell = Uint256::from(2_000u64);
    let repay_loan_action = RepayLoanAction::RepayLoanAndSellAterra {
        aterra_amount_to_sell,
        repay_loan_amount,
    };
    let response = repay_loan_action.to_response(&config).unwrap();

    let repay_stable_coin = Coin {
        denom: STABLE_DENOM.to_string(),
        amount: repay_loan_amount.into(),
    };
    let expected_response = Response::new()
        .add_submessages(vec![
            SubMsg::reply_on_success(
                WasmMsg::Execute {
                    contract_addr: ANCHOR_MARKET_CONTRACT.to_string(),
                    msg: to_binary(&AnchorMarketMsg::RepayStable {}).unwrap(),
                    funds: vec![repay_stable_coin],
                },
                SubmsgIds::RepayLoan.id(),
            ),
            SubMsg::reply_on_success(
                WasmMsg::Execute {
                    contract_addr: ATERRA_TOKEN.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Send {
                        contract: ANCHOR_MARKET_CONTRACT.to_string(),
                        amount: aterra_amount_to_sell.into(),
                        msg: to_binary(&AnchorMarketCw20Msg::RedeemStable {}).unwrap(),
                    })
                    .unwrap(),
                    funds: vec![],
                },
                SubmsgIds::RedeemStableOnRepayLoan.id(),
            ),
        ])
        .add_attributes(vec![
            ("action_1", "repay_loan"),
            ("loan_amount", &repay_loan_amount.to_string()),
            ("action_2", "sell_aterra"),
            ("aterra_amount", &aterra_amount_to_sell.to_string()),
        ]);
    assert_eq!(response, expected_response);
}
