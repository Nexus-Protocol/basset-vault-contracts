use crate::utils::AfterBorrowAction;

use super::sdk::Sdk;
use crate::{
    state::load_config,
    tests::sdk::{ANCHOR_MARKET_CONTRACT, STABLE_DENOM},
};
use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{to_binary, Coin, Response, WasmMsg};

use basset_vault::querier::AnchorMarketMsg;

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

    let expected_response = Response::new()
        .add_message(WasmMsg::Execute {
            contract_addr: ANCHOR_MARKET_CONTRACT.to_string(),
            msg: to_binary(&AnchorMarketMsg::DepositStable {}).unwrap(),
            funds: vec![Coin {
                denom: STABLE_DENOM.to_string(),
                amount: deposit_amount.into(),
            }],
        })
        .add_attributes(vec![
            ("action", "deposit"),
            ("amount", &deposit_amount.to_string()),
        ]);
    assert_eq!(response, expected_response);
}
