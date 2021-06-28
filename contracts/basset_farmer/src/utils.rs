use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{attr, to_binary, Coin, CosmosMsg, ReplyOn, Response, SubMsg, Uint128, WasmMsg};

use crate::state::Config;
use crate::{ContractResult, SubmsgIds};
use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;
use yield_optimizer::{
    psi_distributor::{
        AnyoneMsg as PsiDistributorAnyoneMsg, ExecuteMsg as PsiDistributorExecuteMsg,
    },
    querier::{AnchorMarketCw20Msg, AnchorMarketMsg},
    terraswap::{Asset, AssetInfo},
    terraswap_pair::ExecuteMsg as TerraswapExecuteMsg,
    TaxInfo,
};

#[derive(PartialEq, Eq, Debug)]
pub enum RepayLoanAction {
    RepayLoanAndSellAterra {
        repay_loan_amount: Uint256,
        aterra_amount_to_sell: Uint256,
    },
    SellAterra {
        amount: Uint256,
    },
    RepayLoan {
        amount: Uint256,
    },
    Nothing,
}

//TODO: add tests for 'to_response' method
impl RepayLoanAction {
    pub fn repaying_loan_amount(&self) -> Uint256 {
        match self {
            &RepayLoanAction::RepayLoan { amount } => amount,
            &RepayLoanAction::RepayLoanAndSellAterra {
                repay_loan_amount, ..
            } => repay_loan_amount,

            _ => Uint256::zero(),
        }
    }

    pub fn to_response(&self, config: &Config) -> ContractResult<Response> {
        match self {
            RepayLoanAction::Nothing => Ok(Response::default()),

            &RepayLoanAction::RepayLoan { amount } => {
                let repay_stable_coin = Coin {
                    denom: config.stable_denom.to_string(),
                    amount: amount.into(),
                };

                Ok(Response {
                    messages: vec![],
                    submessages: vec![SubMsg {
                        msg: WasmMsg::Execute {
                            contract_addr: config.anchor_market_contract.to_string(),
                            msg: to_binary(&AnchorMarketMsg::RepayStable {})?,
                            send: vec![repay_stable_coin],
                        }
                        .into(),
                        gas_limit: None,
                        id: SubmsgIds::RepayLoan.id(),
                        reply_on: ReplyOn::Success,
                    }],
                    attributes: vec![attr("action", "repay_loan"), attr("amount", amount)],
                    data: None,
                })
            }

            &RepayLoanAction::SellAterra { amount } => Ok(Response {
                messages: vec![],
                submessages: vec![SubMsg {
                    msg: WasmMsg::Execute {
                        contract_addr: config.aterra_token.to_string(),
                        msg: to_binary(&Cw20ExecuteMsg::Send {
                            contract: config.anchor_market_contract.to_string(),
                            amount: amount.into(),
                            msg: to_binary(&AnchorMarketCw20Msg::RedeemStable {})?,
                        })?,
                        send: vec![],
                    }
                    .into(),
                    gas_limit: None,
                    id: SubmsgIds::RedeemStableOnRepayLoan.id(),
                    //Always because Anchor can block withdrawing
                    //if there are too many borrowers
                    reply_on: ReplyOn::Always,
                }],
                attributes: vec![attr("action", "sell_aterra"), attr("amount", amount)],
                data: None,
            }),

            &RepayLoanAction::RepayLoanAndSellAterra {
                repay_loan_amount,
                aterra_amount_to_sell,
            } => {
                let repay_stable_coin = Coin {
                    denom: config.stable_denom.to_string(),
                    amount: repay_loan_amount.into(),
                };

                Ok(Response {
                    messages: vec![],
                    submessages: vec![
                        SubMsg {
                            //first message is to repay loan
                            msg: WasmMsg::Execute {
                                contract_addr: config.anchor_market_contract.to_string(),
                                msg: to_binary(&AnchorMarketMsg::RepayStable {})?,
                                send: vec![repay_stable_coin],
                            }
                            .into(),
                            gas_limit: None,
                            id: SubmsgIds::RepayLoan.id(),
                            reply_on: ReplyOn::Success,
                        },
                        SubMsg {
                            msg: WasmMsg::Execute {
                                contract_addr: config.aterra_token.to_string(),
                                msg: to_binary(&Cw20ExecuteMsg::Send {
                                    contract: config.anchor_market_contract.to_string(),
                                    amount: aterra_amount_to_sell.into(),
                                    msg: to_binary(&AnchorMarketCw20Msg::RedeemStable {})?,
                                })?,
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
                })
            }
        }
    }
}

//TODO: refactor maybe?
pub fn get_repay_loan_action(
    stable_coin_balance: Uint256,
    aterra_balance: Uint256,
    aterra_exchange_rate: Decimal256,
    total_repay_amount: Uint256,
    aim_buffer_size: Uint256,
    tax_info: &TaxInfo,
    is_first_try: bool,
) -> RepayLoanAction {
    if aterra_balance.is_zero() && stable_coin_balance.is_zero() {
        return RepayLoanAction::Nothing;
    }

    //add tax to repay_amount
    let wanted_stables = calc_wanted_stablecoins(
        stable_coin_balance,
        tax_info.append_tax(total_repay_amount),
        aim_buffer_size,
    );

    let max_amount_to_send = tax_info.subtract_tax(stable_coin_balance);
    let repay_amount = total_repay_amount.min(max_amount_to_send);

    if wanted_stables.is_zero() {
        if repay_amount.is_zero() {
            return RepayLoanAction::Nothing;
        } else {
            return RepayLoanAction::RepayLoan {
                amount: repay_amount,
            };
        }
    }

    //anchor will pay tax, so we will recieve lesser then assume
    let aterra_value = tax_info.subtract_tax(aterra_balance * aterra_exchange_rate);
    //on first try only selling aterra, cause with high probability we will repay loan
    //in 'reply' handler, so don't need to do that twice
    if is_first_try || repay_amount.is_zero() {
        if aterra_value.is_zero() {
            return RepayLoanAction::RepayLoan {
                amount: repay_amount,
            };
        } else {
            let stables_from_aterra_sell = aterra_value.min(wanted_stables);

            let aterra_to_sell =
                tax_info.append_tax(stables_from_aterra_sell) / aterra_exchange_rate;
            //make sure that we do not redeem more then we have (in case if some issue with tax precision)
            let aterra_to_sell = aterra_to_sell.min(aterra_balance);
            return RepayLoanAction::SellAterra {
                amount: aterra_to_sell,
            };
        }
    }

    if aterra_value.is_zero() {
        return RepayLoanAction::RepayLoan {
            amount: repay_amount,
        };
    } else {
        let repay_amount_with_tax = tax_info.append_tax(repay_amount);
        let stables_after_repaying = stable_coin_balance - repay_amount_with_tax;
        if stables_after_repaying >= aim_buffer_size {
            return RepayLoanAction::RepayLoan {
                amount: repay_amount,
            };
        } else {
            //it is not first try, so we are in error handling
            //that means we can't sell more aterra than loan repaid

            let stables_to_fill_buffer = aim_buffer_size - stables_after_repaying;
            let stables_to_repay_loan_remainder =
                tax_info.append_tax(total_repay_amount - repay_amount);
            let total_stables_needed = stables_to_repay_loan_remainder + stables_to_fill_buffer;
            let loan_amoun_that_will_be_repayed = tax_info.subtract_tax(repay_amount);
            let bounded_aterra_value = loan_amoun_that_will_be_repayed.min(total_stables_needed);
            //adding tax that anchor contract will pay to send stable coins to us
            let aterra_to_sell = tax_info.append_tax(bounded_aterra_value) / aterra_exchange_rate;
            //make sure that we do not redeem more then we have (in case if some issue with tax precision)
            let aterra_to_sell = aterra_to_sell.min(aterra_balance);
            RepayLoanAction::RepayLoanAndSellAterra {
                aterra_amount_to_sell: aterra_to_sell,
                repay_loan_amount: repay_amount,
            }
        }
    }
}

fn calc_wanted_stablecoins(
    stable_coin_balance: Uint256,
    repay_amount: Uint256,
    aim_buffer_size: Uint256,
) -> Uint256 {
    //we have enough balance to repay loan without any additional stables
    if stable_coin_balance >= repay_amount + aim_buffer_size {
        return Uint256::zero();
    }

    //we can take some coins from buffer to repay loan
    if stable_coin_balance >= aim_buffer_size {
        let can_get_from_balance = stable_coin_balance - aim_buffer_size;
        if repay_amount <= can_get_from_balance {
            return Uint256::zero();
        }
        return repay_amount - can_get_from_balance;
    }

    //need to fill up buffer and repay loan
    let add_to_buffer = aim_buffer_size - stable_coin_balance;
    return repay_amount + add_to_buffer;
}

#[derive(PartialEq, Eq, Debug)]
pub enum AfterBorrowAction {
    Deposit { amount: Uint256 },
    Nothing,
}

impl AfterBorrowAction {
    pub fn to_response(&self, config: &Config) -> ContractResult<Response> {
        match self {
            AfterBorrowAction::Nothing => Ok(Response::default()),

            &AfterBorrowAction::Deposit { amount } => Ok(Response {
                messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: config.anchor_market_contract.to_string(),
                    msg: to_binary(&AnchorMarketMsg::DepositStable {})?,
                    send: vec![Coin {
                        denom: config.stable_denom.to_string(),
                        amount: amount.into(),
                    }],
                })],
                submessages: vec![],
                attributes: vec![attr("action", "deposit"), attr("amount", amount)],
                data: None,
            }),
        }
    }
}

pub fn calc_after_borrow_action(
    stable_coin_balance: Uint256,
    aim_buf_size: Uint256,
    tax_info: &TaxInfo,
) -> AfterBorrowAction {
    if aim_buf_size >= stable_coin_balance {
        return AfterBorrowAction::Nothing;
    }

    let accessible_amount = stable_coin_balance - aim_buf_size;
    AfterBorrowAction::Deposit {
        amount: tax_info.subtract_tax(accessible_amount),
    }
}

#[derive(Debug, PartialEq)]
pub enum ActionWithProfit {
    BuyPsi {
        amount: Uint256,
    },
    DepositToAnc {
        amount: Uint256,
    },
    Split {
        buy_psi: Uint256,
        deposit_to_anc: Uint256,
    },
    Nothing,
}

impl ActionWithProfit {
    pub fn to_response(&self, config: &Config, tax_info: &TaxInfo) -> ContractResult<Response> {
        match self {
            ActionWithProfit::Nothing => Ok(Response {
                messages: vec![],
                submessages: vec![],
                attributes: vec![
                    attr("action", "distribute_rewards"),
                    attr("rewards_profit", "zero"),
                ],
                data: None,
            }),

            &ActionWithProfit::DepositToAnc { amount } => {
                let stable_coin_to_lending: Uint128 = tax_info.subtract_tax(amount).into();

                Ok(Response {
                    messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.anchor_market_contract.to_string(),
                        msg: to_binary(&AnchorMarketMsg::DepositStable {})?,
                        send: vec![Coin {
                            denom: config.stable_denom.clone(),
                            amount: stable_coin_to_lending,
                        }],
                    })],
                    submessages: vec![],
                    attributes: vec![
                        attr("action", "distribute_rewards"),
                        attr("deposit_to_anc", stable_coin_to_lending),
                    ],
                    data: None,
                })
            }

            &ActionWithProfit::BuyPsi { amount } => {
                let stable_coin_to_buy_psi: Uint128 = tax_info.subtract_tax(amount).into();
                let swap_asset = Asset {
                    info: AssetInfo::NativeToken {
                        denom: config.stable_denom.clone(),
                    },
                    amount: stable_coin_to_buy_psi,
                };

                Ok(Response {
                    messages: vec![
                        CosmosMsg::Wasm(WasmMsg::Execute {
                            contract_addr: config.psi_stable_swap_contract.to_string(),
                            msg: to_binary(&TerraswapExecuteMsg::Swap {
                                offer_asset: swap_asset,
                                max_spread: None,
                                belief_price: None,
                                to: Some(config.psi_distributor_addr.to_string()),
                            })?,
                            send: vec![Coin {
                                denom: config.stable_denom.clone(),
                                amount: stable_coin_to_buy_psi,
                            }],
                        }),
                        CosmosMsg::Wasm(WasmMsg::Execute {
                            contract_addr: config.psi_distributor_addr.to_string(),
                            msg: to_binary(&PsiDistributorExecuteMsg::Anyone {
                                anyone_msg: PsiDistributorAnyoneMsg::DistributeRewards,
                            })?,
                            send: vec![],
                        }),
                    ],
                    submessages: vec![],
                    attributes: vec![
                        attr("action", "distribute_rewards"),
                        attr("bying_psi", stable_coin_to_buy_psi),
                    ],
                    data: None,
                })
            }

            &ActionWithProfit::Split {
                buy_psi,
                deposit_to_anc,
            } => {
                let stable_coin_to_lending: Uint128 = tax_info.subtract_tax(deposit_to_anc).into();
                let stable_coin_to_buy_psi: Uint128 = tax_info.subtract_tax(buy_psi).into();
                let swap_asset = Asset {
                    info: AssetInfo::NativeToken {
                        denom: config.stable_denom.clone(),
                    },
                    amount: stable_coin_to_buy_psi,
                };

                Ok(Response {
                    messages: vec![
                        CosmosMsg::Wasm(WasmMsg::Execute {
                            contract_addr: config.anchor_market_contract.to_string(),
                            msg: to_binary(&AnchorMarketMsg::DepositStable {})?,
                            send: vec![Coin {
                                denom: config.stable_denom.clone(),
                                amount: stable_coin_to_lending,
                            }],
                        }),
                        CosmosMsg::Wasm(WasmMsg::Execute {
                            contract_addr: config.psi_stable_swap_contract.to_string(),
                            msg: to_binary(&TerraswapExecuteMsg::Swap {
                                offer_asset: swap_asset,
                                max_spread: None,
                                belief_price: None,
                                to: Some(config.psi_distributor_addr.to_string()),
                            })?,
                            send: vec![Coin {
                                denom: config.stable_denom.clone(),
                                amount: stable_coin_to_buy_psi,
                            }],
                        }),
                        CosmosMsg::Wasm(WasmMsg::Execute {
                            contract_addr: config.psi_distributor_addr.to_string(),
                            msg: to_binary(&PsiDistributorExecuteMsg::Anyone {
                                anyone_msg: PsiDistributorAnyoneMsg::DistributeRewards,
                            })?,
                            send: vec![],
                        }),
                    ],
                    submessages: vec![],
                    attributes: vec![
                        attr("action", "distribute_rewards"),
                        attr("bying_psi", stable_coin_to_buy_psi),
                        attr("deposit_to_anc", stable_coin_to_lending),
                    ],
                    data: None,
                })
            }
        }
    }
}

pub fn split_profit_to_handle_interest(
    borrowed_amount: Uint256,
    aterra_amount: Uint256,
    aterra_exchange_rate: Decimal256,
    stable_coin_balance: Uint256,
    stable_coin_balance_before_sell_anc: Uint256,
    over_loan_balance_value: Decimal256,
) -> ActionWithProfit {
    if stable_coin_balance <= stable_coin_balance_before_sell_anc {
        return ActionWithProfit::Nothing;
    }

    let total_stable_coin_balance = aterra_amount * aterra_exchange_rate + stable_coin_balance;
    let selling_anc_profit = stable_coin_balance - stable_coin_balance_before_sell_anc;

    let aim_stable_balance = borrowed_amount * over_loan_balance_value;
    if aim_stable_balance <= total_stable_coin_balance {
        return ActionWithProfit::BuyPsi {
            amount: selling_anc_profit,
        };
    }

    let amount_to_anc_deposit = aim_stable_balance - total_stable_coin_balance;
    if selling_anc_profit <= amount_to_anc_deposit {
        return ActionWithProfit::DepositToAnc {
            amount: selling_anc_profit,
        };
    }

    let buy_psi_amount = selling_anc_profit - amount_to_anc_deposit;
    return ActionWithProfit::Split {
        buy_psi: buy_psi_amount,
        deposit_to_anc: amount_to_anc_deposit,
    };
}

pub fn is_anc_rewards_claimable(
    current_height: u64,
    last_rewards_claiming_height: u64,
    claiming_rewards_delay: u64,
) -> bool {
    current_height > last_rewards_claiming_height
        && (current_height - last_rewards_claiming_height) >= claiming_rewards_delay
}

#[cfg(test)]
mod test {
    use super::{
        calc_after_borrow_action, calc_wanted_stablecoins, get_repay_loan_action,
        split_profit_to_handle_interest, ActionWithProfit, AfterBorrowAction, RepayLoanAction,
    };

    use cosmwasm_bignumber::{Decimal256, Uint256};
    use std::str::FromStr;
    use yield_optimizer::TaxInfo;

    #[test]
    fn get_repay_loan_action_sell_all_1() {
        let aterra_balance = Uint256::from(500u64);
        let stable_coin_balance = Uint256::from(100u64);
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let repay_amount = Uint256::from(1_000u64);
        let aim_buffer_size = Uint256::from(100u64);
        let tax_info = TaxInfo {
            rate: Decimal256::percent(1),
            cap: Uint256::from(100u64),
        };
        let repay_action = get_repay_loan_action(
            stable_coin_balance,
            aterra_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
            &tax_info,
            true,
        );
        assert_eq!(
            RepayLoanAction::SellAterra {
                amount: aterra_balance
            },
            repay_action
        );
    }

    #[test]
    fn get_repay_loan_action_do_nothing_1() {
        let aterra_balance = Uint256::zero();
        let stable_coin_balance = Uint256::zero();
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let repay_amount = Uint256::from(1_000u64);
        let aim_buffer_size = Uint256::from(100u64);
        let tax_info = TaxInfo {
            rate: Decimal256::percent(1),
            cap: Uint256::from(100u64),
        };
        let repay_action = get_repay_loan_action(
            stable_coin_balance,
            aterra_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
            &tax_info,
            true,
        );
        assert_eq!(RepayLoanAction::Nothing, repay_action);
    }

    #[test]
    fn get_repay_loan_action_do_nothing_2() {
        let aterra_balance = Uint256::zero();
        let stable_coin_balance = Uint256::from(1_000u64);
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let repay_amount = Uint256::zero();
        let aim_buffer_size = Uint256::from(100u64);
        let tax_info = TaxInfo {
            rate: Decimal256::percent(1),
            cap: Uint256::from(100u64),
        };
        let repay_action = get_repay_loan_action(
            stable_coin_balance,
            aterra_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
            &tax_info,
            true,
        );
        assert_eq!(RepayLoanAction::Nothing, repay_action);
    }

    #[test]
    fn get_repay_loan_action_do_nothing_3() {
        let aterra_balance = Uint256::from(2_000u64);
        let stable_coin_balance = Uint256::from(100u64);
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let repay_amount = Uint256::zero();
        let aim_buffer_size = Uint256::from(100u64);
        let tax_info = TaxInfo {
            rate: Decimal256::percent(1),
            cap: Uint256::from(100u64),
        };
        let repay_action = get_repay_loan_action(
            stable_coin_balance,
            aterra_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
            &tax_info,
            true,
        );
        assert_eq!(RepayLoanAction::Nothing, repay_action);
    }

    #[test]
    fn get_repay_loan_action_do_sell_aterra_for_fee() {
        let aterra_balance = Uint256::from(2_000u64);
        let stable_coin_balance = Uint256::from(2_000_000u64);
        let aterra_exchange_rate = Decimal256::from_str("1.25").unwrap();
        let repay_amount = Uint256::from(1_000_000u64);
        let aim_buffer_size = Uint256::from(1_000_000u64);
        let tax_info = TaxInfo {
            rate: Decimal256::percent(1),
            cap: Uint256::from(1000u64),
        };
        let repay_action = get_repay_loan_action(
            stable_coin_balance,
            aterra_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
            &tax_info,
            true,
        );
        //need to sell aterra to get fees for repaying 1kk (which is 1000)
        //(1000+10 for fee) / 1.25 = 808
        assert_eq!(
            RepayLoanAction::SellAterra {
                amount: Uint256::from(808u64)
            },
            repay_action
        );
    }

    #[test]
    fn get_repay_loan_action_repay_stable_coins() {
        let aterra_balance = Uint256::zero();
        let stable_coin_balance = Uint256::from(200u64);
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let repay_amount = Uint256::from(1_000u64);
        let aim_buffer_size = Uint256::from(100u64);
        let tax_info = TaxInfo {
            rate: Decimal256::percent(1),
            cap: Uint256::from(100u64),
        };
        let repay_action = get_repay_loan_action(
            stable_coin_balance,
            aterra_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
            &tax_info,
            true,
        );
        assert_eq!(
            RepayLoanAction::RepayLoan {
                amount: tax_info.subtract_tax(stable_coin_balance)
            },
            repay_action
        );
    }

    #[test]
    fn get_repay_loan_action_sell_all_2() {
        let aterra_balance = Uint256::from(200u64);
        let stable_coin_balance = Uint256::zero();
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let repay_amount = Uint256::from(1_000u64);
        let aim_buffer_size = Uint256::from(100u64);
        let tax_info = TaxInfo {
            rate: Decimal256::percent(1),
            cap: Uint256::from(100u64),
        };
        let repay_action = get_repay_loan_action(
            stable_coin_balance,
            aterra_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
            &tax_info,
            true,
        );
        assert_eq!(
            RepayLoanAction::SellAterra {
                amount: aterra_balance
            },
            repay_action
        );
    }

    #[test]
    fn get_repay_loan_action_fill_aim_buffer() {
        let aterra_balance = Uint256::from(100u64);
        let stable_coin_balance = Uint256::from(100u64);
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let repay_amount = Uint256::from(200u64);
        //total: 100+120
        //total - repay = 220 - 200 = 20 which is less then we need to aim_buffer_size, so sell all
        let aim_buffer_size = Uint256::from(100u64);
        let tax_info = TaxInfo {
            rate: Decimal256::percent(1),
            cap: Uint256::from(100u64),
        };
        let repay_action = get_repay_loan_action(
            stable_coin_balance,
            aterra_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
            &tax_info,
            true,
        );
        assert_eq!(
            RepayLoanAction::SellAterra {
                amount: aterra_balance
            },
            repay_action
        );
    }

    #[test]
    fn get_repay_loan_action_no_need_to_sell_aterra() {
        let aterra_balance = Uint256::from(100u64);
        let stable_coin_balance = Uint256::from(100u64);
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let repay_amount = Uint256::from(50u64);
        let aim_buffer_size = Uint256::from(50u64);
        //ust_buffer >= repay + aim_buffer
        //so no need to sell
        let tax_info = TaxInfo {
            rate: Decimal256::zero(),
            cap: Uint256::from(100u64),
        };
        let repay_action = get_repay_loan_action(
            stable_coin_balance,
            aterra_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
            &tax_info,
            true,
        );
        assert_eq!(
            RepayLoanAction::RepayLoan {
                amount: repay_amount
            },
            repay_action
        );
    }

    #[test]
    fn get_repay_loan_action_need_to_sell_cause_of_tax() {
        let aterra_balance = Uint256::from(1000u64);
        let stable_coin_balance = Uint256::from(1000u64);
        let aterra_exchange_rate = Decimal256::from_str("1.25").unwrap();
        let repay_amount = Uint256::from(680u64);
        let aim_buffer_size = Uint256::from(330u64);
        //ust_buffer = repay + aim_buffer
        //but in order to repay you need to pay tax
        //so not enough balance to do that
        let tax_info = TaxInfo {
            rate: Decimal256::percent(25),
            cap: Uint256::from(999999u64),
        };
        let repay_action = get_repay_loan_action(
            stable_coin_balance,
            aterra_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
            &tax_info,
            true,
        );
        assert_eq!(
            RepayLoanAction::SellAterra {
                //to repay loan you need to send 680ust, but to send 680ust you need to pay 170:
                //680 + 170 + 330(buffer) = 1180
                //but you have only 1000, so need to sell aterra for 180
                //180 + 180*0.25 = 225(ust amount to receive from anchor for selling aterra)
                //225 - 225/1.25 = 180
                amount: Uint256::from(180u64)
            },
            repay_action
        );
    }

    #[test]
    fn get_repay_loan_action_get_portion_from_buffer() {
        let aterra_balance = Uint256::from(1000u64);
        let aterra_exchange_rate = Decimal256::from_str("1.1").unwrap();
        let stable_coin_balance = Uint256::from(1000u64);
        let repay_amount = Uint256::from(800u64);
        let aim_buffer_size = Uint256::from(825u64);
        //we can get 175 coins from buffer, cause aim_buffer < current_buffer
        //so need to sell aterra only to get 800 - 175 = 625 coins, plus 100 for tax = 725
        let tax_info = TaxInfo {
            rate: Decimal256::percent(20),
            cap: Uint256::from(100u64),
        };
        let repay_action = get_repay_loan_action(
            stable_coin_balance,
            aterra_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
            &tax_info,
            true,
        );
        assert_eq!(
            //725/1.1 + 100 = 750
            RepayLoanAction::SellAterra {
                amount: Uint256::from(750u64),
            },
            repay_action
        );
    }

    #[test]
    fn get_repay_loan_action_sell_to_add_to_buffer() {
        let aterra_balance = Uint256::from(1000u64);
        let stable_coin_balance = Uint256::from(100u64);
        let aterra_exchange_rate = Decimal256::from_str("1.25").unwrap();
        let repay_amount = Uint256::from(70u64);
        let aim_buffer_size = Uint256::from(120u64);
        let tax_info = TaxInfo {
            rate: Decimal256::percent(99),
            cap: Uint256::from(10u64),
        };
        let repay_action = get_repay_loan_action(
            stable_coin_balance,
            aterra_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
            &tax_info,
            true,
        );
        assert_eq!(
            RepayLoanAction::SellAterra {
                //70 to repay with 10 tax: 80 coins
                //120(aim_buffer) - (100(balance) - 80) = 100
                //so sell aterra to get 100:
                //(100 +10)/1.25 = 88
                amount: Uint256::from(88u64),
            },
            repay_action
        );
    }

    #[test]
    fn get_repay_loan_action_repay_from_buffer() {
        let aterra_balance = Uint256::zero();
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let repay_amount = Uint256::from(20u64);
        let stable_coin_balance = Uint256::from(100u64);
        let aim_buffer_size = Uint256::from(100u64);
        let tax_info = TaxInfo {
            rate: Decimal256::percent(10),
            cap: Uint256::from(15u64),
        };
        let repay_action = get_repay_loan_action(
            stable_coin_balance,
            aterra_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
            &tax_info,
            true,
        );
        assert_eq!(
            RepayLoanAction::RepayLoan {
                amount: repay_amount
            },
            repay_action
        );
    }

    #[test]
    fn get_repay_loan_action_do_nothing_not_a_first_try() {
        let aterra_balance = Uint256::zero();
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let repay_amount = Uint256::zero();
        let stable_coin_balance = Uint256::from(100u64);
        let aim_buffer_size = Uint256::from(100u64);
        let tax_info = TaxInfo {
            rate: Decimal256::percent(10),
            cap: Uint256::from(15u64),
        };
        let repay_action = get_repay_loan_action(
            stable_coin_balance,
            aterra_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
            &tax_info,
            false,
        );
        assert_eq!(RepayLoanAction::Nothing, repay_action);
    }

    #[test]
    fn get_repay_loan_action_sell_aterra_only() {
        let aterra_balance = Uint256::from(1000u64);
        let aterra_exchange_rate = Decimal256::from_str("0.88").unwrap();
        let repay_amount = Uint256::from(200u64);
        let stable_coin_balance = Uint256::from(100u64);
        let aim_buffer_size = Uint256::from(100u64);
        let tax_info = TaxInfo {
            rate: Decimal256::percent(10),
            cap: Uint256::from(150u64),
        };
        let repay_action = get_repay_loan_action(
            stable_coin_balance,
            aterra_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
            &tax_info,
            true,
        );
        assert_eq!(
            RepayLoanAction::SellAterra {
                //we need to sell aterra to get stables for repay loan + tax
                //plus tax that anchor pays to transfer stable to us
                //so: 200 + 200*0.1 = 220 - amount that we need to repay loan
                //to get it we need to sell aterra for: 220 + 220*0.1 = 242
                //but aterra price is 0.88, so 242/0.88 = 275
                amount: Uint256::from(275u64)
            },
            repay_action
        );
    }

    #[test]
    fn get_repay_loan_action_do_not_need_stables_just_repay_loan() {
        let aterra_balance = Uint256::from(300_000u64);
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let stable_coin_balance = Uint256::from(100_000u64);
        let repay_amount = Uint256::from(1_000u64);
        let aim_buffer_size = Uint256::from(8_000u64);
        let tax_info = TaxInfo {
            rate: Decimal256::percent(99),
            cap: Uint256::from(750u64),
        };
        let repay_action = get_repay_loan_action(
            stable_coin_balance,
            aterra_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
            &tax_info,
            false,
        );
        assert_eq!(
            RepayLoanAction::RepayLoan {
                amount: repay_amount,
            },
            repay_action
        );
    }

    #[test]
    fn get_repay_loan_action_sell_entire_buffer_and_calc_fee_to_repay_loan_remainder() {
        let aterra_balance = Uint256::from(300_000u64);
        let aterra_exchange_rate = Decimal256::from_str("1.25").unwrap();
        let stable_coin_balance = Uint256::from(100_000u64);
        let repay_amount = Uint256::from(100_000u64);
        let aim_buffer_size = Uint256::from(80_000u64);
        let tax_info = TaxInfo {
            rate: Decimal256::percent(88),
            cap: Uint256::from(750u64),
        };
        let repay_action = get_repay_loan_action(
            stable_coin_balance,
            aterra_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
            &tax_info,
            false,
        );
        assert_eq!(
            //repaying loan at limit: 99_250
            //remainder to repay loan: 750
            //to fill buffer: 80k
            //we have: 0 (after repayment)
            //total: 80k + repay_remainder(750) + tax_to_repay_remainder(660) + 750(fee to receive from anchor)
            //total: 82_160
            RepayLoanAction::RepayLoanAndSellAterra {
                repay_loan_amount: Uint256::from(99_250u64), //minus 750 tax cap
                aterra_amount_to_sell: Uint256::from(65_728u64), //82_160 / 1.25
            },
            repay_action
        );
    }

    #[test]
    fn get_repay_loan_action_calc_fees_to_repay_loan() {
        let aterra_balance = Uint256::from(300_000u64);
        let aterra_exchange_rate = Decimal256::from_str("1.25").unwrap();
        let stable_coin_balance = Uint256::from(10_000u64);
        let repay_amount = Uint256::from(100_000u64);
        let aim_buffer_size = Uint256::from(80_000u64);
        let tax_info = TaxInfo {
            rate: Decimal256::from_str("0.007").unwrap(),
            cap: Uint256::from(750u64),
        };
        let repay_action = get_repay_loan_action(
            stable_coin_balance,
            aterra_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
            &tax_info,
            true,
        );
        assert_eq!(
            //to repay loan: 100k
            //to fill buffer: 80k
            //we have: 10k
            //total: 100k + 80k - 10k + fee to pay loan (700) + 750(fee to receive from anchor)
            //total: 171_450
            RepayLoanAction::SellAterra {
                amount: Uint256::from(137160u64), //171_450 / 1.25
            },
            repay_action
        );
    }

    #[test]
    fn get_repay_loan_action_last_aterra_sell() {
        let aterra_balance = Uint256::from(20_000u64);
        let aterra_exchange_rate = Decimal256::from_str("1.25").unwrap();
        let stable_coin_balance = Uint256::from(100_000u64);
        let repay_amount = Uint256::from(20_000u64);
        let aim_buffer_size = Uint256::from(80_000u64);
        let tax_info = TaxInfo {
            rate: Decimal256::percent(90),
            cap: Uint256::from(750u64),
        };
        let repay_action = get_repay_loan_action(
            stable_coin_balance,
            aterra_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
            &tax_info,
            false,
        );
        assert_eq!(
            RepayLoanAction::RepayLoanAndSellAterra {
                repay_loan_amount: repay_amount,
                //repay for 20k, but pay 750 tax
                //so need to sell aterra for 750, to make aim_buffer
                //plus 750 for anchor to pay tax
                // (750 + 750*0.9) / 1.25 = 1140
                aterra_amount_to_sell: Uint256::from(1_140u64)
            },
            repay_action
        );
    }

    #[test]
    fn get_repay_loan_action_sell_aterra_only_for_repayed_amount() {
        let aterra_balance = Uint256::from(1_000_000u64);
        let aterra_exchange_rate = Decimal256::from_str("1.25").unwrap();
        let stable_coin_balance = Uint256::from(100_000u64);
        let repay_amount = Uint256::from(200_000u64);
        let aim_buffer_size = Uint256::from(1_000_000u64);
        let tax_info = TaxInfo {
            rate: Decimal256::percent(90),
            cap: Uint256::from(750u64),
        };
        let repay_action = get_repay_loan_action(
            stable_coin_balance,
            aterra_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
            &tax_info,
            false,
        );
        let repayed_loan_amoun = stable_coin_balance - tax_info.cap;
        assert_eq!(
            RepayLoanAction::RepayLoanAndSellAterra {
                repay_loan_amount: repayed_loan_amoun,
                //CAN'T SELL ATERRA TO GET 100K!!!
                //because this is case where you repaying loan to be able to redeem aterra
                //BUT tax will be payed by anchor contract, so you can sell aterra only for
                //amount that was repayed: 100k - 750 = 99_250
                //99_250 / 1.25 = 79_400
                aterra_amount_to_sell: Uint256::from(79_400u64)
            },
            repay_action
        );
    }

    #[test]
    fn get_repay_loan_action_last_aterra_sell_but_without_loan_repaying() {
        let aterra_balance = Uint256::from(20_000u64);
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let stable_coin_balance = Uint256::from(40_000u64);
        let repay_amount = Uint256::zero();
        let aim_buffer_size = Uint256::from(80_000u64);
        let tax_info = TaxInfo {
            rate: Decimal256::percent(1),
            cap: Uint256::from(750u64),
        };
        let repay_action = get_repay_loan_action(
            stable_coin_balance,
            aterra_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
            &tax_info,
            false,
        );
        assert_eq!(
            RepayLoanAction::SellAterra {
                //need to add 40k to buffer, but we have only 20k aterra which is not enough
                //so sell all
                amount: aterra_balance,
            },
            repay_action
        );
    }

    #[test]
    fn get_repay_loan_action_not_enough_aterra_to_fill_buffer_after_repayment() {
        let aterra_balance = Uint256::from(100u64);
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let stable_coin_balance = Uint256::from(100_000u64);
        let repay_amount = Uint256::from(20_000u64);
        let aim_buffer_size = Uint256::from(80_000u64);
        let tax_info = TaxInfo {
            rate: Decimal256::percent(99),
            cap: Uint256::from(750u64),
        };
        let repay_action = get_repay_loan_action(
            stable_coin_balance,
            aterra_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
            &tax_info,
            false,
        );
        assert_eq!(
            RepayLoanAction::RepayLoanAndSellAterra {
                repay_loan_amount: repay_amount,
                //we spent 20k to repay loan + 750 to tax
                //so need to sell aterra for 750, but we have only 100, so sell all
                aterra_amount_to_sell: aterra_balance
            },
            repay_action
        );
    }

    #[test]
    fn get_repay_loan_action_repay_loan_only() {
        let aterra_balance = Uint256::zero();
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let stable_coin_balance = Uint256::from(100_000u64);
        let repay_amount = Uint256::from(1_000u64);
        let aim_buffer_size = Uint256::from(80_000u64);
        let tax_info = TaxInfo {
            rate: Decimal256::percent(1),
            cap: Uint256::from(750u64),
        };
        let repay_action = get_repay_loan_action(
            stable_coin_balance,
            aterra_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
            &tax_info,
            false,
        );
        assert_eq!(
            RepayLoanAction::RepayLoan {
                amount: repay_amount,
            },
            repay_action
        );
    }

    #[test]
    fn get_repay_loan_action_repay_loan_only_not_enough_balance_for_fee() {
        let aterra_balance = Uint256::zero();
        let aterra_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let stable_coin_balance = Uint256::from(1_000u64);
        let repay_amount = Uint256::from(1_000u64);
        let aim_buffer_size = Uint256::from(80_000u64);
        let tax_info = TaxInfo {
            rate: Decimal256::percent(25),
            cap: Uint256::from(750u64),
        };
        let repay_action = get_repay_loan_action(
            stable_coin_balance,
            aterra_balance,
            aterra_exchange_rate,
            repay_amount,
            aim_buffer_size,
            &tax_info,
            false,
        );
        assert_eq!(
            //need to repay 1k of loan, but we dont have enough cause of fee
            //so send all
            RepayLoanAction::RepayLoan {
                amount: tax_info.subtract_tax(stable_coin_balance),
            },
            repay_action
        );
    }

    #[test]
    fn calc_wanted_stablecoins_1() {
        let stable_coin_balance = Uint256::zero();
        let repay_amount = Uint256::zero();
        let aim_buffer_size = Uint256::zero();
        let result = calc_wanted_stablecoins(stable_coin_balance, repay_amount, aim_buffer_size);
        assert!(result.is_zero())
    }

    #[test]
    fn calc_wanted_stablecoins_2() {
        let stable_coin_balance = Uint256::from(100u64);
        let repay_amount = Uint256::zero();
        let aim_buffer_size = Uint256::zero();
        let result = calc_wanted_stablecoins(stable_coin_balance, repay_amount, aim_buffer_size);
        assert!(result.is_zero())
    }

    #[test]
    fn calc_wanted_stablecoins_3() {
        let stable_coin_balance = Uint256::zero();
        let repay_amount = Uint256::from(100u64);
        let aim_buffer_size = Uint256::zero();
        let result = calc_wanted_stablecoins(stable_coin_balance, repay_amount, aim_buffer_size);
        assert_eq!(repay_amount, result)
    }

    #[test]
    fn calc_wanted_stablecoins_4() {
        let stable_coin_balance = Uint256::zero();
        let repay_amount = Uint256::zero();
        let aim_buffer_size = Uint256::from(100u64);
        let result = calc_wanted_stablecoins(stable_coin_balance, repay_amount, aim_buffer_size);
        assert_eq!(aim_buffer_size, result)
    }

    #[test]
    fn calc_wanted_stablecoins_5() {
        let stable_coin_balance = Uint256::zero();
        let repay_amount = Uint256::from(100u64);
        let aim_buffer_size = Uint256::from(100u64);
        let result = calc_wanted_stablecoins(stable_coin_balance, repay_amount, aim_buffer_size);
        assert_eq!(repay_amount + aim_buffer_size, result)
    }

    #[test]
    fn calc_wanted_stablecoins_6() {
        let stable_coin_balance = Uint256::from(200u64);
        let repay_amount = Uint256::from(100u64);
        let aim_buffer_size = Uint256::from(100u64);
        let result = calc_wanted_stablecoins(stable_coin_balance, repay_amount, aim_buffer_size);
        assert!(result.is_zero())
    }

    #[test]
    fn calc_wanted_stablecoins_7() {
        let stable_coin_balance = Uint256::from(200u64);
        let repay_amount = Uint256::from(120u64);
        let aim_buffer_size = Uint256::from(100u64);
        let result = calc_wanted_stablecoins(stable_coin_balance, repay_amount, aim_buffer_size);
        assert_eq!(
            aim_buffer_size - (stable_coin_balance - repay_amount),
            result
        )
    }

    #[test]
    fn calc_wanted_stablecoins_8() {
        let stable_coin_balance = Uint256::from(20u64);
        let repay_amount = Uint256::zero();
        let aim_buffer_size = Uint256::from(100u64);
        let result = calc_wanted_stablecoins(stable_coin_balance, repay_amount, aim_buffer_size);
        assert_eq!(aim_buffer_size - stable_coin_balance, result)
    }

    #[test]
    fn calc_wanted_stablecoins_9() {
        let stable_coin_balance = Uint256::from(20u64);
        let repay_amount = Uint256::from(50u64);
        let aim_buffer_size = Uint256::zero();
        let result = calc_wanted_stablecoins(stable_coin_balance, repay_amount, aim_buffer_size);
        assert_eq!(repay_amount - stable_coin_balance, result)
    }

    #[test]
    fn calc_wanted_stablecoins_10() {
        let stable_coin_balance = Uint256::from(20u64);
        let repay_amount = Uint256::from(50u64);
        let aim_buffer_size = Uint256::from(50u64);
        let result = calc_wanted_stablecoins(stable_coin_balance, repay_amount, aim_buffer_size);
        assert_eq!(repay_amount + aim_buffer_size - stable_coin_balance, result)
    }

    #[test]
    fn calc_wanted_stablecoins_11() {
        let stable_coin_balance = Uint256::from(200u64);
        let repay_amount = Uint256::from(120u64);
        let aim_buffer_size = Uint256::from(50u64);
        let result = calc_wanted_stablecoins(stable_coin_balance, repay_amount, aim_buffer_size);
        assert!(result.is_zero())
    }

    #[test]
    fn calc_after_borrow_action_zeroes() {
        let tax_info = TaxInfo {
            rate: Decimal256::percent(1),
            cap: Uint256::from(750u64),
        };
        let action = calc_after_borrow_action(Uint256::zero(), Uint256::zero(), &tax_info);
        assert_eq!(AfterBorrowAction::Nothing, action)
    }

    #[test]
    fn calc_after_borrow_action_balance_bigger_than_zero_buffer() {
        let tax_info = TaxInfo {
            rate: Decimal256::percent(1),
            cap: Uint256::from(750u64),
        };
        let balance = Uint256::from(1_000_000u64);
        let action = calc_after_borrow_action(balance, Uint256::zero(), &tax_info);
        assert_eq!(
            AfterBorrowAction::Deposit {
                amount: balance - tax_info.cap
            },
            action
        )
    }

    #[test]
    fn calc_after_borrow_action_balance_bigger_than_buffer() {
        let tax_info = TaxInfo {
            rate: Decimal256::percent(1),
            cap: Uint256::from(750u64),
        };

        let balance = Uint256::from(1_000_000u64);
        let buffer = Uint256::from(1_000u64);
        let action = calc_after_borrow_action(balance, buffer, &tax_info);
        assert_eq!(
            AfterBorrowAction::Deposit {
                amount: tax_info.subtract_tax(balance - buffer)
            },
            action
        )
    }

    #[test]
    fn calc_after_borrow_action_buffer_bigger_than_zero_balance() {
        let tax_info = TaxInfo {
            rate: Decimal256::percent(1),
            cap: Uint256::from(750u64),
        };

        let buffer = Uint256::from(1_000_000u64);
        let action = calc_after_borrow_action(Uint256::zero(), buffer, &tax_info);
        assert_eq!(AfterBorrowAction::Nothing, action)
    }

    #[test]
    fn calc_after_borrow_action_buffer_bigger_than_balance() {
        let tax_info = TaxInfo {
            rate: Decimal256::percent(1),
            cap: Uint256::from(750u64),
        };

        let balance = Uint256::from(1_000u64);
        let buffer = Uint256::from(1_000_000u64);
        let action = calc_after_borrow_action(balance, buffer, &tax_info);
        assert_eq!(AfterBorrowAction::Nothing, action)
    }

    #[test]
    fn split_profit_to_handle_interest_zero_profit() {
        let borrowed_amount = Uint256::from(2_000u64);
        let aterra_balance = Uint256::from(1_500u64);
        let aterra_state_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let stable_coin_balance = Uint256::from(500u64);
        let stable_coin_balance_before_sell_anc = Uint256::from(500u64);
        let over_loan_balance_value = Decimal256::from_str("1.01").unwrap();

        let action_with_profit = split_profit_to_handle_interest(
            borrowed_amount,
            aterra_balance,
            aterra_state_exchange_rate,
            stable_coin_balance,
            stable_coin_balance_before_sell_anc,
            over_loan_balance_value,
        );

        assert_eq!(ActionWithProfit::Nothing, action_with_profit);
    }

    #[test]
    fn split_profit_to_handle_interest_negative_profit() {
        let borrowed_amount = Uint256::from(2_000u64);
        let aterra_balance = Uint256::from(1_500u64);
        let aterra_state_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let stable_coin_balance = Uint256::from(500u64);
        let stable_coin_balance_before_sell_anc = Uint256::from(1_000u64);
        let over_loan_balance_value = Decimal256::from_str("1.01").unwrap();

        let action_with_profit = split_profit_to_handle_interest(
            borrowed_amount,
            aterra_balance,
            aterra_state_exchange_rate,
            stable_coin_balance,
            stable_coin_balance_before_sell_anc,
            over_loan_balance_value,
        );

        assert_eq!(ActionWithProfit::Nothing, action_with_profit);
    }

    #[test]
    fn split_profit_to_handle_interest_current_balance_is_bigger_than_aim() {
        let borrowed_amount = Uint256::from(2_000u64);
        //1500*1.2 = 1800
        let aterra_balance = Uint256::from(1_500u64);
        let aterra_state_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let stable_coin_balance = Uint256::from(800u64);
        let stable_coin_balance_before_sell_anc = Uint256::from(500u64);
        let over_loan_balance_value = Decimal256::from_str("1.01").unwrap();

        let action_with_profit = split_profit_to_handle_interest(
            borrowed_amount,
            aterra_balance,
            aterra_state_exchange_rate,
            stable_coin_balance,
            stable_coin_balance_before_sell_anc,
            over_loan_balance_value,
        );

        assert_eq!(
            ActionWithProfit::BuyPsi {
                amount: Uint256::from(300u64)
            },
            action_with_profit
        );
    }

    #[test]
    fn split_profit_to_handle_interest_current_balance_is_lesser_than_aim_plus_profit() {
        let borrowed_amount = Uint256::from(2_000u64);
        //1500*1.2 = 1800
        let aterra_balance = Uint256::from(1_500u64);
        let aterra_state_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let stable_coin_balance = Uint256::from(100u64);
        let stable_coin_balance_before_sell_anc = Uint256::from(50u64);
        //so, aim is 2020
        let over_loan_balance_value = Decimal256::from_str("1.01").unwrap();

        let action_with_profit = split_profit_to_handle_interest(
            borrowed_amount,
            aterra_balance,
            aterra_state_exchange_rate,
            stable_coin_balance,
            stable_coin_balance_before_sell_anc,
            over_loan_balance_value,
        );

        assert_eq!(
            ActionWithProfit::DepositToAnc {
                amount: Uint256::from(50u64)
            },
            action_with_profit
        );
    }

    #[test]
    fn split_profit_to_handle_interest_current_balance_is_lesser_than_aim_but_profit_helps() {
        let borrowed_amount = Uint256::from(2_000u64);
        //1500*1.2 = 1800
        let aterra_balance = Uint256::from(1_500u64);
        let aterra_state_exchange_rate = Decimal256::from_str("1.2").unwrap();
        let stable_coin_balance = Uint256::from(200u64);
        let stable_coin_balance_before_sell_anc = Uint256::from(150u64);
        //so, aim is 2020
        let over_loan_balance_value = Decimal256::from_str("1.01").unwrap();

        let action_with_profit = split_profit_to_handle_interest(
            borrowed_amount,
            aterra_balance,
            aterra_state_exchange_rate,
            stable_coin_balance,
            stable_coin_balance_before_sell_anc,
            over_loan_balance_value,
        );

        assert_eq!(
            ActionWithProfit::Split {
                buy_psi: Uint256::from(30u64),
                deposit_to_anc: Uint256::from(20u64),
            },
            action_with_profit
        );
    }
}
