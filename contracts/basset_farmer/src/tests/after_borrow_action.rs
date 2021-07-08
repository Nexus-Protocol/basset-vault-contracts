use crate::{
    state::{load_repaying_loan_state, store_config, RepayingLoanState},
    utils::{AfterBorrowAction, RepayLoanAction},
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
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    attr,
    testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR},
    CosmosMsg,
};
use cosmwasm_std::{
    to_binary, Addr, Coin, ContractResult, Decimal, Reply, ReplyOn, Response, SubMsg,
    SubcallResponse, Uint128, WasmMsg,
};
use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;
use std::str::FromStr;
use yield_optimizer::{
    basset_farmer::{AnyoneMsg, ExecuteMsg},
    basset_farmer_config::BorrowerActionResponse,
    querier::{
        AnchorMarketCw20Msg, AnchorMarketEpochStateResponse, AnchorMarketMsg, AnchorOverseerMsg,
        BorrowerInfoResponse,
    },
};

#[test]
fn after_borrow_action_to_response_nothing() {
    let sdk = Sdk::init();
    let config = load_config(sdk.deps.as_ref().storage).unwrap();

    let after_borrow_action = AfterBorrowAction::Nothing;
    let response = after_borrow_action.to_response(&config).unwrap();
    assert_eq!(response, Response::default());
}

#[test]
fn after_borrow_action_to_response_deposit() {
    let sdk = Sdk::init();
    let config = load_config(sdk.deps.as_ref().storage).unwrap();

    let deposit_amount = Uint256::from(2_000u64);
    let after_borrow_action = AfterBorrowAction::Deposit {
        amount: deposit_amount,
    };
    let response = after_borrow_action.to_response(&config).unwrap();

    let expected_response = Response {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: ANCHOR_MARKET_CONTRACT.to_string(),
            msg: to_binary(&AnchorMarketMsg::DepositStable {}).unwrap(),
            send: vec![Coin {
                denom: STABLE_DENOM.to_string(),
                amount: deposit_amount.into(),
            }],
        })],
        submessages: vec![],
        attributes: vec![attr("action", "deposit"), attr("amount", deposit_amount)],
        data: None,
    };
    assert_eq!(response, expected_response);
}
