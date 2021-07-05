use crate::{
    state::{load_repaying_loan_state, store_config, RepayingLoanState},
    SubmsgIds, TOO_HIGH_BORROW_DEMAND_ERR_MSG,
};

use super::sdk::Sdk;
use crate::{
    state::{load_child_contracts_info, load_config, ChildContractsInfo, Config},
    tests::sdk::{
        ANCHOR_CUSTODY_BASSET_CONTRACT, ANCHOR_MARKET_CONTRACT, ANCHOR_OVERSEER_CONTRACT,
        ANCHOR_TOKEN, ANC_STABLE_SWAP_CONTRACT, ATERRA_TOKEN, BASSET_FARMER_CONFIG_CONTRACT,
        BASSET_TOKEN_ADDR, CLAIMING_REWARDS_DELAY, COLLATERAL_TOKEN_SYMBOL, GOVERNANCE_CONTRACT,
        NASSET_CONTRACT_ADDR, NASSET_TOKEN_CODE_ID, NASSET_TOKEN_CONFIG_HOLDER_CODE_ID,
        NASSET_TOKEN_CONFIG_HOLDER_CONTRACT, NASSET_TOKEN_REWARDS_CODE_ID,
        NASSET_TOKEN_REWARDS_CONTRACT, OVER_LOAN_BALANCE_VALUE, PSI_DISTRIBUTOR_CODE_ID,
        PSI_DISTRIBUTOR_CONTRACT, PSI_STABLE_SWAP_CONTRACT, PSI_TOKEN, STABLE_DENOM,
    },
};
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
    let mut sdk = Sdk::init();

    let stable_coin_balance = Uint128::from(200u64);
    let loan_to_repay = Uint256::from(10_000u64);
    let locked_basset_amount = Uint128::from(10_000u64);
    let basset_farmer_loan_amount = Uint256::from(10_000u64);
    let advised_buffer_size = Uint256::from(50u64);
    let aterra_balance = Uint256::from(200u64);
    let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();

    sdk.set_collateral_balance(locked_basset_amount);
    sdk.set_aterra_balance(aterra_balance);
    sdk.set_loan(basset_farmer_loan_amount);
    sdk.set_borrower_action(BorrowerActionResponse::Repay {
        amount: loan_to_repay,
        advised_buffer_size,
    });
    sdk.set_aterra_exchange_rate(aterra_exchange_rate);
    sdk.set_tax(20, 10u128);

    // -= REBALANCE =-
    {
        let response = sdk.rebalance().unwrap();
        assert_eq!(
            response.submessages,
            vec![SubMsg {
                msg: WasmMsg::Execute {
                    contract_addr: ATERRA_TOKEN.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Send {
                        contract: ANCHOR_MARKET_CONTRACT.to_string(),
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
        let rapaying_state = load_repaying_loan_state(&sdk.deps.storage).unwrap();
        assert_eq!(rapaying_state.iteration_index, 0);
        assert_eq!(rapaying_state.to_repay_amount, loan_to_repay);
        assert_eq!(rapaying_state.repaying_amount, Uint256::zero());
        assert_eq!(rapaying_state.aim_buffer_size, advised_buffer_size);
    }

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
            response.submessages,
            vec![SubMsg {
                msg: WasmMsg::Execute {
                    contract_addr: ANCHOR_MARKET_CONTRACT.to_string(),
                    msg: to_binary(&AnchorMarketMsg::RepayStable {}).unwrap(),
                    send: vec![repay_stable_coin.clone()],
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

    let stable_coin_initial_balance = Uint128(5_000);
    sdk.set_stable_balance(stable_coin_initial_balance);
    sdk.set_aterra_exchange_rate(Decimal256::from_str("1.25").unwrap());
    let aterra_balance = Uint256::from(7_000u64);
    sdk.set_aterra_balance(aterra_balance);

    //no tax
    sdk.set_tax(0, 99999999999u128);

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
            response.submessages,
            vec![SubMsg {
                msg: WasmMsg::Execute {
                    contract_addr: ATERRA_TOKEN.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Send {
                        contract: ANCHOR_MARKET_CONTRACT.to_string(),
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
    }

    // -= REDEEM failed =-
    {
        let response = sdk.aterra_redeed_failed().unwrap();
        //now contract should repay loan with buffer and try to redeem aterra for that amount
        assert_eq!(
            response.submessages,
            vec![
                SubMsg {
                    msg: WasmMsg::Execute {
                        contract_addr: ANCHOR_MARKET_CONTRACT.to_string(),
                        msg: to_binary(&AnchorMarketMsg::RepayStable {}).unwrap(),
                        send: vec![Coin {
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
                        send: vec![],
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
            response.submessages,
            vec![
                SubMsg {
                    msg: WasmMsg::Execute {
                        contract_addr: ANCHOR_MARKET_CONTRACT.to_string(),
                        msg: to_binary(&AnchorMarketMsg::RepayStable {}).unwrap(),
                        send: vec![Coin {
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
                        send: vec![],
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
