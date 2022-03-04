use crate::{state::{load_repaying_loan_state, load_after_deposit_action}, SubmsgIds, tests::sdk::{BASSET_TOKEN_ADDR, ANCHOR_CUSTODY_BASSET_CONTRACT, ANCHOR_OVERSEER_CONTRACT}};

use super::{sdk::Sdk, after_borrow_action};
use crate::tests::sdk::{ANCHOR_MARKET_CONTRACT, ATERRA_TOKEN, STABLE_DENOM};
use cosmwasm_bignumber::{Decimal256, Uint256};

use basset_vault::{
    basset_vault_strategy::BorrowerActionResponse,
    querier::{AnchorMarketCw20Msg, AnchorMarketMsg, AnchorCustodyCw20Msg, AnchorOverseerMsg},
    BASSET_VAULT_LOAN_REPAYMENT_MAX_RECURSION_DEEP,
};
use cosmwasm_std::{to_binary, Coin, Decimal, ReplyOn, Response, SubMsg, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;
use std::convert::Into;
use std::str::FromStr;

#[test]
fn deposit_bassets_to_anchor_in_rebalance() {
    let mut sdk = Sdk::init();

    let basset_balance = Uint256::from(1_000_000_u64);
    let amount_to_borrow = Uint256::from(50_000_u64);
    let after_deposit_action = BorrowerActionResponse::Borrow {
        amount: amount_to_borrow,
        advised_buffer_size: Uint256::from(50u64),
    };

    sdk.set_basset_balance(basset_balance);
    sdk.set_borrower_action(BorrowerActionResponse::Deposit {
        deposit_amount: basset_balance,
        action_after: Box::new(after_deposit_action.clone()),
    });

    let response = sdk.rebalance().unwrap();
    assert_eq!(
        response.messages,
        vec![
            SubMsg::new(WasmMsg::Execute {
                contract_addr: BASSET_TOKEN_ADDR.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Send {
                    contract: ANCHOR_CUSTODY_BASSET_CONTRACT.to_string(),
                    amount: basset_balance.into(),
                    msg: to_binary(&AnchorCustodyCw20Msg::DepositCollateral {}).unwrap(),
                }).unwrap(),
                funds: vec![],
            }),
            SubMsg::reply_on_success(
                WasmMsg::Execute {
                    contract_addr: ANCHOR_OVERSEER_CONTRACT.to_string(),
                    msg: to_binary(&AnchorOverseerMsg::LockCollateral {
                        collaterals: vec![(BASSET_TOKEN_ADDR.to_string(), basset_balance)],
                    }).unwrap(),
                    funds: vec![],
                },
                SubmsgIds::AfterDepositAction.id(),
            ),
        ]
    );

    let loaded_after_deposit_action = load_after_deposit_action(&sdk.deps.storage).unwrap();
    assert_eq!(loaded_after_deposit_action, after_deposit_action);

    let after_deposit_action_response = sdk.after_deposit_action().unwrap();

    assert_eq!(
        after_deposit_action_response.messages,
        vec![SubMsg::reply_on_success(
            WasmMsg::Execute {
                contract_addr: ANCHOR_MARKET_CONTRACT.to_string(),
                msg: to_binary(&AnchorMarketMsg::BorrowStable {
                    borrow_amount: amount_to_borrow,
                    to: None,
                }).unwrap(),
                funds: vec![],
            },
            SubmsgIds::Borrowing.id(),
        )],
    );
}

#[test]
fn deposit_incorrect_amount_of_bassets_to_anchor_in_rebalance() {
    let mut sdk = Sdk::init();

    let basset_balance = Uint256::from(1_000_000_u64);
    let deposit_amount = Uint256::from(2_000_000_u64);

    sdk.set_basset_balance(basset_balance);
    sdk.set_borrower_action(BorrowerActionResponse::Deposit {
        deposit_amount,
        action_after: Box::new(BorrowerActionResponse::Nothing {}),
    });

    assert!(sdk.rebalance().is_err());
}

#[test]
fn deposit_zero_amount_of_bassets_to_anchor_in_rebalance() {
    let mut sdk = Sdk::init();

    let basset_balance = Uint256::from(1_000_000_u64);
    let deposit_amount = Uint256::from(0u64);

    sdk.set_basset_balance(basset_balance);
    sdk.set_borrower_action(BorrowerActionResponse::Deposit {
        deposit_amount,
        action_after: Box::new(BorrowerActionResponse::Nothing {}),
    });

    assert!(sdk.rebalance().is_err());
}
