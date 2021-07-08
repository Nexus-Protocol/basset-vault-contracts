use crate::{
    state::{load_repaying_loan_state, store_config, RepayingLoanState},
    utils::RepayLoanAction,
    SubmsgIds, TOO_HIGH_BORROW_DEMAND_ERR_MSG,
};

use super::sdk::Sdk;
use crate::{
    state::{load_child_contracts_info, load_config, ChildContractsInfo, Config},
    tests::sdk::{
        ANCHOR_CUSTODY_BASSET_CONTRACT, ANCHOR_MARKET_CONTRACT, ANCHOR_OVERSEER_CONTRACT,
        ANCHOR_TOKEN, ANC_STABLE_SWAP_CONTRACT, ATERRA_TOKEN, BASSET_FARMER_CONFIG_CONTRACT,
        BASSET_TOKEN_ADDR, CLAIMING_REWARDS_DELAY, COLLATERAL_TOKEN_SYMBOL, GOVERNANCE_CONTRACT,
        NASSET_TOKEN_ADDR, NASSET_TOKEN_CODE_ID, NASSET_TOKEN_CONFIG_HOLDER_CODE_ID,
        NASSET_TOKEN_CONFIG_HOLDER_CONTRACT, NASSET_TOKEN_REWARDS_CODE_ID,
        NASSET_TOKEN_REWARDS_CONTRACT, OVER_LOAN_BALANCE_VALUE, PSI_DISTRIBUTOR_CODE_ID,
        PSI_DISTRIBUTOR_CONTRACT, PSI_STABLE_SWAP_CONTRACT, PSI_TOKEN, STABLE_DENOM,
    },
};
use cosmwasm_bignumber::{Uint256};
use cosmwasm_std::{
    attr,
};
use cosmwasm_std::{
    to_binary, Addr, Coin, ContractResult, Decimal, Reply, ReplyOn, Response, SubMsg,
    SubcallResponse, Uint128, WasmMsg,
};
use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;

use yield_optimizer::{
    basset_farmer::{AnyoneMsg, ExecuteMsg},
    basset_farmer_config::BorrowerActionResponse,
    querier::{
        AnchorMarketCw20Msg, AnchorMarketEpochStateResponse, AnchorMarketMsg, AnchorOverseerMsg,
        BorrowerInfoResponse,
    },
};

#[test]
fn repay_loan_action_to_response_nothing() {
    let sdk = Sdk::init();
    let config = load_config(sdk.deps.as_ref().storage).unwrap();

    let repay_loan_action = RepayLoanAction::Nothing;
    let response = repay_loan_action.to_response(&config).unwrap();
    assert_eq!(response, Response::default());
}

#[test]
fn repay_loan_action_to_response_repay_loan() {
    let sdk = Sdk::init();
    let config = load_config(sdk.deps.as_ref().storage).unwrap();

    let repay_amount = Uint256::from(2_000u64);
    let repay_loan_action = RepayLoanAction::RepayLoan {
        amount: repay_amount,
    };
    let response = repay_loan_action.to_response(&config).unwrap();

    let repay_stable_coin = Coin {
        denom: STABLE_DENOM.to_string(),
        amount: repay_amount.into(),
    };
    let expected_response = Response {
        messages: vec![],
        submessages: vec![SubMsg {
            msg: WasmMsg::Execute {
                contract_addr: ANCHOR_MARKET_CONTRACT.to_string(),
                msg: to_binary(&AnchorMarketMsg::RepayStable {}).unwrap(),
                send: vec![repay_stable_coin],
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
    let config = load_config(sdk.deps.as_ref().storage).unwrap();

    let sell_amount = Uint256::from(2_000u64);
    let repay_loan_action = RepayLoanAction::SellAterra {
        amount: sell_amount,
    };
    let response = repay_loan_action.to_response(&config).unwrap();

    let expected_response = Response {
        messages: vec![],
        submessages: vec![SubMsg {
            msg: WasmMsg::Execute {
                contract_addr: ATERRA_TOKEN.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Send {
                    contract: ANCHOR_MARKET_CONTRACT.to_string(),
                    amount: sell_amount.into(),
                    msg: to_binary(&AnchorMarketCw20Msg::RedeemStable {}).unwrap(),
                })
                .unwrap(),
                send: vec![],
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
    let config = load_config(sdk.deps.as_ref().storage).unwrap();

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
    let expected_response = Response {
        messages: vec![],
        submessages: vec![
            SubMsg {
                msg: WasmMsg::Execute {
                    contract_addr: ANCHOR_MARKET_CONTRACT.to_string(),
                    msg: to_binary(&AnchorMarketMsg::RepayStable {}).unwrap(),
                    send: vec![repay_stable_coin],
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
                        msg: to_binary(&AnchorMarketCw20Msg::RedeemStable {}).unwrap(),
                    })
                    .unwrap(),
                    send: vec![],
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
