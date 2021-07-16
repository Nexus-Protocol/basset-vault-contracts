use crate::{state::query_external_config, utils::RepayLoanAction, SubmsgIds};

use super::sdk::Sdk;
use crate::tests::sdk::{ANCHOR_MARKET_CONTRACT, ATERRA_TOKEN, STABLE_DENOM};
use cosmwasm_bignumber::Uint256;
use cosmwasm_std::attr;
use cosmwasm_std::{to_binary, Coin, ReplyOn, Response, SubMsg, WasmMsg};
use cw20::Cw20ExecuteMsg;

use basset_vault::querier::{AnchorMarketCw20Msg, AnchorMarketMsg};

#[test]
fn repay_loan_action_to_response_nothing() {
    let sdk = Sdk::init();
    let external_config = query_external_config(sdk.deps.as_ref()).unwrap();

    let repay_loan_action = RepayLoanAction::Nothing;
    let response = repay_loan_action.to_response(&external_config).unwrap();
    assert_eq!(response, Response::default());
}

#[test]
fn repay_loan_action_to_response_repay_loan() {
    let sdk = Sdk::init();
    let external_config = query_external_config(sdk.deps.as_ref()).unwrap();

    let repay_amount = Uint256::from(2_000u64);
    let repay_loan_action = RepayLoanAction::RepayLoan {
        amount: repay_amount,
    };
    let response = repay_loan_action.to_response(&external_config).unwrap();

    let repay_stable_coin = Coin {
        denom: STABLE_DENOM.to_string(),
        amount: repay_amount.into(),
    };
    let expected_response = Response {
        events: vec![],
        messages: vec![SubMsg {
            msg: WasmMsg::Execute {
                contract_addr: ANCHOR_MARKET_CONTRACT.to_string(),
                msg: to_binary(&AnchorMarketMsg::RepayStable).unwrap(),
                funds: vec![repay_stable_coin],
            }
            .into(),
            gas_limit: None,
            id: SubmsgIds::RepayLoan.id(),
            reply_on: ReplyOn::Success,
        }],
        attributes: vec![attr("action", "repay_loan"), attr("amount", repay_amount)],
        data: None,
    };
    assert_eq!(response, expected_response);
}

#[test]
fn repay_loan_action_to_response_sell_aterra() {
    let sdk = Sdk::init();
    let external_config = query_external_config(sdk.deps.as_ref()).unwrap();

    let sell_amount = Uint256::from(2_000u64);
    let repay_loan_action = RepayLoanAction::SellAterra {
        amount: sell_amount,
    };
    let response = repay_loan_action.to_response(&external_config).unwrap();

    let expected_response = Response {
        events: vec![],
        messages: vec![SubMsg {
            msg: WasmMsg::Execute {
                contract_addr: ATERRA_TOKEN.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Send {
                    contract: ANCHOR_MARKET_CONTRACT.to_string(),
                    amount: sell_amount.into(),
                    msg: to_binary(&AnchorMarketCw20Msg::RedeemStable).unwrap(),
                })
                .unwrap(),
                funds: vec![],
            }
            .into(),
            gas_limit: None,
            id: SubmsgIds::RedeemStableOnRepayLoan.id(),
            reply_on: ReplyOn::Always,
        }],
        attributes: vec![attr("action", "sell_aterra"), attr("amount", sell_amount)],
        data: None,
    };
    assert_eq!(response, expected_response);
}

#[test]
fn repay_loan_action_to_response_repay_loan_and_sell_aterra() {
    let sdk = Sdk::init();
    let external_config = query_external_config(sdk.deps.as_ref()).unwrap();

    let repay_loan_amount = Uint256::from(5_000u64);
    let aterra_amount_to_sell = Uint256::from(2_000u64);
    let repay_loan_action = RepayLoanAction::RepayLoanAndSellAterra {
        aterra_amount_to_sell,
        repay_loan_amount,
    };
    let response = repay_loan_action.to_response(&external_config).unwrap();

    let repay_stable_coin = Coin {
        denom: STABLE_DENOM.to_string(),
        amount: repay_loan_amount.into(),
    };
    let expected_response = Response {
        events: vec![],
        messages: vec![
            SubMsg {
                msg: WasmMsg::Execute {
                    contract_addr: ANCHOR_MARKET_CONTRACT.to_string(),
                    msg: to_binary(&AnchorMarketMsg::RepayStable).unwrap(),
                    funds: vec![repay_stable_coin],
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
                        amount: aterra_amount_to_sell.into(),
                        msg: to_binary(&AnchorMarketCw20Msg::RedeemStable).unwrap(),
                    })
                    .unwrap(),
                    funds: vec![],
                }
                .into(),
                gas_limit: None,
                id: SubmsgIds::RedeemStableOnRepayLoan.id(),
                reply_on: ReplyOn::Success,
            },
        ],
        attributes: vec![
            attr("action_1", "repay_loan"),
            attr("loan_amount", repay_loan_amount),
            attr("action_2", "sell_aterra"),
            attr("aterra_amount", aterra_amount_to_sell),
        ],
        data: None,
    };
    assert_eq!(response, expected_response);
}
