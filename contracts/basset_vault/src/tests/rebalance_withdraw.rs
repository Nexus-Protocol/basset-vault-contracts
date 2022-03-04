use crate::{state::{load_repaying_loan_state, load_after_deposit_action}, SubmsgIds, tests::sdk::{BASSET_TOKEN_ADDR, ANCHOR_CUSTODY_BASSET_CONTRACT, ANCHOR_OVERSEER_CONTRACT}};

use super::{sdk::Sdk, after_borrow_action};
use crate::tests::sdk::{ANCHOR_MARKET_CONTRACT, ATERRA_TOKEN, STABLE_DENOM};
use cosmwasm_bignumber::{Decimal256, Uint256};

use basset_vault::{
    basset_vault_strategy::BorrowerActionResponse,
    querier::{AnchorMarketCw20Msg, AnchorMarketMsg, AnchorCustodyCw20Msg, AnchorOverseerMsg, AnchorCustodyMsg},
    BASSET_VAULT_LOAN_REPAYMENT_MAX_RECURSION_DEEP,
};
use cosmwasm_std::{to_binary, Coin, Decimal, ReplyOn, Response, SubMsg, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;
use std::convert::Into;
use std::str::FromStr;

#[test]
fn repay_all_and_withdraw_in_rebalance() {
    let mut sdk = Sdk::init();

    let collateral_balance = Uint256::from(100_000_u64);
    let loan = Uint256::from(10_000_u64);
    let stable_coin_balance = Uint128::from(20_000u64);

    sdk.set_collateral_balance(collateral_balance);
    sdk.set_loan(loan);
    sdk.set_stable_balance(stable_coin_balance);
    sdk.set_borrower_action(BorrowerActionResponse::RepayAllAndWithdraw {
        withdraw_amount: collateral_balance,
    });

    let response = sdk.rebalance().unwrap();

    let repay_stable_coin = Coin {
        denom: STABLE_DENOM.to_string(),
        amount: loan.into(),
    };

    assert_eq!(
        response.messages,
        vec![
            SubMsg::reply_on_success(
                WasmMsg::Execute {
                    contract_addr: ANCHOR_MARKET_CONTRACT.to_string(),
                    msg: to_binary(&AnchorMarketMsg::RepayStable {}).unwrap(),
                    funds: vec![repay_stable_coin],
                },
                SubmsgIds::RepayLoan.id(),
            ),
            SubMsg::new(WasmMsg::Execute {
                contract_addr: ANCHOR_OVERSEER_CONTRACT.to_string(),
                msg: to_binary(&AnchorOverseerMsg::UnlockCollateral {
                    collaterals: vec![(BASSET_TOKEN_ADDR.to_string(), collateral_balance)],
                }).unwrap(),
                funds: vec![],
            }),
            SubMsg::new(WasmMsg::Execute {
                contract_addr: ANCHOR_CUSTODY_BASSET_CONTRACT.to_string(),
                msg: to_binary(&AnchorCustodyMsg::WithdrawCollateral {
                    amount: Some(collateral_balance),
                }).unwrap(),
                funds: vec![],
            })
        ]
    );
}

#[test]
fn repay_all_and_withdraw_zero_in_rebalance() {
    let mut sdk = Sdk::init();

    let collateral_balance = Uint256::from(100_000_u64);
    let loan = Uint256::from(10_000_u64);
    let stable_coin_balance = Uint128::from(20_000u64);

    sdk.set_collateral_balance(collateral_balance);
    sdk.set_loan(loan);
    sdk.set_stable_balance(stable_coin_balance);
    sdk.set_borrower_action(BorrowerActionResponse::RepayAllAndWithdraw {
        withdraw_amount: Uint256::zero(),
    });

    let response = sdk.rebalance().unwrap();

    let repay_stable_coin = Coin {
        denom: STABLE_DENOM.to_string(),
        amount: loan.into(),
    };

    assert_eq!(
        response.messages,
        vec![
            SubMsg::reply_on_success(
                WasmMsg::Execute {
                    contract_addr: ANCHOR_MARKET_CONTRACT.to_string(),
                    msg: to_binary(&AnchorMarketMsg::RepayStable {}).unwrap(),
                    funds: vec![repay_stable_coin],
                },
                SubmsgIds::RepayLoan.id(),
            ),
        ]
    );
}


#[test]
fn try_withdraw_more_than_in_collateral_in_rebalance() {
    let mut sdk = Sdk::init();

    let collateral_balance = Uint256::from(100_000_u64);
    let withdraw_amount = Uint256::from(200_000_u64);
    let loan = Uint256::from(10_000_u64);
    let stable_coin_balance = Uint128::from(20_000u64);

    sdk.set_collateral_balance(collateral_balance);
    sdk.set_loan(loan);
    sdk.set_stable_balance(stable_coin_balance);
    sdk.set_borrower_action(BorrowerActionResponse::RepayAllAndWithdraw {
        withdraw_amount,
    });

    assert!(sdk.rebalance().is_err());
}
