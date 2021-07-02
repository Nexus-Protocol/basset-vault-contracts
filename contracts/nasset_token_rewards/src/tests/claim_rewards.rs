use crate::tests::mock_dependencies;
use crate::{
    error::ContractError,
    state::{load_holder, load_state, Holder, State},
};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR},
    Addr, CosmosMsg, Decimal, StdError, WasmMsg,
};
use cosmwasm_std::{to_binary, Uint128};
use cw20::Cw20ReceiveMsg;
use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;
use std::str::FromStr;
use yield_optimizer::nasset_token_rewards::{AnyoneMsg, ExecuteMsg};

fn initialize_token(token_addr: String) {}

#[test]
fn increase_balance_and_claim_rewards() {
    let psi_token_addr = "addr0001".to_string();
    let nasset_token_addr = "addr0002".to_string();
    let governance_contract_addr = "addr0003".to_string();

    let msg = yield_optimizer::nasset_token_rewards::InstantiateMsg {
        psi_token_addr: psi_token_addr.clone(),
        nasset_token_addr: nasset_token_addr.clone(),
        governance_contract_addr,
    };

    let mut deps = mock_dependencies(&[]);

    // -= INITIALIZATION =-
    {
        let env = mock_env();
        let info = mock_info("addr0010", &[]);
        crate::contract::instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();
    }

    //first user comes, increase his balance
    let user_1_address = Addr::unchecked("addr9999".to_string());
    let deposit_1_amount: Uint128 = 100u128.into();
    //rewards already there
    let rewards_before_receive_nasset: Uint128 = 1000u64.into();
    let rewards_after_receive_nasset: Uint128 = 5000u64.into();
    {
        deps.querier.with_token_balances(&[(
            &psi_token_addr,
            &[(
                &MOCK_CONTRACT_ADDR.to_string(),
                &rewards_before_receive_nasset,
            )],
        )]);

        // -= nAsset token send message to increase user balance =-
        let user_1_increase_balance =
            yield_optimizer::nasset_token_rewards::TokenMsg::IncreaseBalance {
                address: user_1_address.to_string(),
                amount: deposit_1_amount,
            };
        let info = mock_info(&nasset_token_addr, &vec![]);
        let res = crate::contract::execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Token {
                token_msg: user_1_increase_balance,
            },
        );
        assert!(res.is_ok());
        let holder_state = load_holder(&deps.storage, &user_1_address).unwrap();
        assert_eq!(deposit_1_amount, holder_state.balance);
        assert_eq!(Decimal::zero(), holder_state.index);
        assert_eq!(Decimal::zero(), holder_state.pending_rewards);
    }

    {
        //rewards incoming
        deps.querier.with_token_balances(&[(
            &psi_token_addr,
            &[(
                &MOCK_CONTRACT_ADDR.to_string(),
                &(rewards_before_receive_nasset + rewards_after_receive_nasset),
            )],
        )]);
        let update_index_msg = yield_optimizer::nasset_token_rewards::AnyoneMsg::UpdateGlobalIndex;
        let info = mock_info(&user_1_address.to_string(), &vec![]);
        let res = crate::contract::execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Anyone {
                anyone_msg: update_index_msg,
            },
        );
        assert!(res.is_ok());
    }

    {
        // -= USER SEND CLAIM message =-
        let claim_msg =
            yield_optimizer::nasset_token_rewards::AnyoneMsg::ClaimRewards { recipient: None };
        let info = mock_info(&user_1_address.to_string(), &vec![]);
        let res = crate::contract::execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Anyone {
                anyone_msg: claim_msg,
            },
        )
        .unwrap();

        assert_eq!(
            res.messages,
            vec![CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: psi_token_addr.clone(),
                send: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: user_1_address.to_string(),
                    amount: rewards_before_receive_nasset + rewards_after_receive_nasset,
                })
                .unwrap(),
            })]
        );
        assert!(res.submessages.is_empty());

        let holder = load_holder(&deps.storage, &user_1_address).unwrap();
        assert_eq!(deposit_1_amount, holder.balance);
        assert_eq!(Decimal::from_str("60").unwrap(), holder.index);
        assert_eq!(Decimal::zero(), holder.pending_rewards);

        let state = load_state(&deps.storage).unwrap();
        assert_eq!(Decimal::from_str("60").unwrap(), state.global_index);
        assert_eq!(deposit_1_amount, state.total_balance);
        assert_eq!(Uint128::zero(), state.prev_reward_balance);
    }
}

#[test]
fn claim_zero_rewards() {
    let psi_token_addr = "addr0001".to_string();
    let nasset_token_addr = "addr0002".to_string();
    let governance_contract_addr = "addr0003".to_string();

    let msg = yield_optimizer::nasset_token_rewards::InstantiateMsg {
        psi_token_addr: psi_token_addr.clone(),
        nasset_token_addr: nasset_token_addr.clone(),
        governance_contract_addr,
    };

    let mut deps = mock_dependencies(&[]);

    // -= INITIALIZATION =-
    {
        let env = mock_env();
        let info = mock_info("addr0010", &[]);
        crate::contract::instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();
    }

    //first user comes, increase his balance
    let user_1_address = Addr::unchecked("addr9999".to_string());
    let deposit_1_amount: Uint128 = 100u128.into();
    //rewards already there
    let rewards: Uint128 = Uint128::zero();
    {
        deps.querier.with_token_balances(&[(
            &psi_token_addr,
            &[(&MOCK_CONTRACT_ADDR.to_string(), &rewards)],
        )]);

        // -= nAsset token send message to increase user balance =-
        let user_1_increase_balance =
            yield_optimizer::nasset_token_rewards::TokenMsg::IncreaseBalance {
                address: user_1_address.to_string(),
                amount: deposit_1_amount,
            };
        let info = mock_info(&nasset_token_addr, &vec![]);
        let res = crate::contract::execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Token {
                token_msg: user_1_increase_balance,
            },
        );
        assert!(res.is_ok());
        let holder_state = load_holder(&deps.storage, &user_1_address).unwrap();
        assert_eq!(deposit_1_amount, holder_state.balance);
        assert_eq!(Decimal::zero(), holder_state.index);
        assert_eq!(Decimal::zero(), holder_state.pending_rewards);
    }

    {
        //rewards incoming
        deps.querier.with_token_balances(&[(
            &psi_token_addr,
            &[(&MOCK_CONTRACT_ADDR.to_string(), &rewards)],
        )]);
        let update_index_msg = yield_optimizer::nasset_token_rewards::AnyoneMsg::UpdateGlobalIndex;
        let info = mock_info(&user_1_address.to_string(), &vec![]);
        let res = crate::contract::execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Anyone {
                anyone_msg: update_index_msg,
            },
        );
        assert!(res.is_ok());
    }

    {
        // -= USER SEND CLAIM message =-
        let claim_msg =
            yield_optimizer::nasset_token_rewards::AnyoneMsg::ClaimRewards { recipient: None };
        let info = mock_info(&user_1_address.to_string(), &vec![]);
        let res = crate::contract::execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Anyone {
                anyone_msg: claim_msg,
            },
        );

        assert!(res.is_err());
        let error = res.err().unwrap();
        if let ContractError::Std(StdError::GenericErr { msg }) = error {
            assert_eq!("No rewards have accrued yet", msg);
        } else {
            panic!("wrong error type");
        };

        let holder = load_holder(&deps.storage, &user_1_address).unwrap();
        assert_eq!(deposit_1_amount, holder.balance);
        assert_eq!(Decimal::zero(), holder.index);
        assert_eq!(Decimal::zero(), holder.pending_rewards);

        let state = load_state(&deps.storage).unwrap();
        assert_eq!(Decimal::zero(), state.global_index);
        assert_eq!(deposit_1_amount, state.total_balance);
        assert_eq!(Uint128::zero(), state.prev_reward_balance);
    }
}

#[test]
fn second_user_comes_after_rewards_already_there() {
    let psi_token_addr = "addr0001".to_string();
    let nasset_token_addr = "addr0002".to_string();
    let governance_contract_addr = "addr0003".to_string();

    let msg = yield_optimizer::nasset_token_rewards::InstantiateMsg {
        psi_token_addr: psi_token_addr.clone(),
        nasset_token_addr: nasset_token_addr.clone(),
        governance_contract_addr,
    };

    let mut deps = mock_dependencies(&[]);

    // -= INITIALIZATION =-
    {
        let env = mock_env();
        let info = mock_info("addr0010", &[]);
        crate::contract::instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();
    }

    //first user comes, increase his balance
    let user_1_address = Addr::unchecked("addr9999".to_string());
    let deposit_1_amount: Uint128 = 100u128.into();
    let user_2_address = Addr::unchecked("addr6666".to_string());
    let deposit_2_amount: Uint128 = 300u128.into();
    //rewards already there
    let rewards_before_receive_nasset: Uint128 = 1000u64.into();
    let rewards_after_receive_nasset: Uint128 = 5000u64.into();
    {
        deps.querier.with_token_balances(&[(
            &psi_token_addr,
            &[(
                &MOCK_CONTRACT_ADDR.to_string(),
                &rewards_before_receive_nasset,
            )],
        )]);

        // -= nAsset token send message to increase first user balance =-
        let user_1_increase_balance =
            yield_optimizer::nasset_token_rewards::TokenMsg::IncreaseBalance {
                address: user_1_address.to_string(),
                amount: deposit_1_amount,
            };
        let info = mock_info(&nasset_token_addr, &vec![]);
        let res = crate::contract::execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Token {
                token_msg: user_1_increase_balance,
            },
        );
        assert!(res.is_ok());
        let holder_state = load_holder(&deps.storage, &user_1_address).unwrap();
        assert_eq!(deposit_1_amount, holder_state.balance);
        assert_eq!(Decimal::zero(), holder_state.index);
        assert_eq!(Decimal::zero(), holder_state.pending_rewards);
    }

    //second user comes, increase his balance
    {
        // -= nAsset token send message to increase second user balance =-
        let user_2_increase_balance =
            yield_optimizer::nasset_token_rewards::TokenMsg::IncreaseBalance {
                address: user_2_address.to_string(),
                amount: deposit_2_amount,
            };
        let info = mock_info(&nasset_token_addr, &vec![]);
        let res = crate::contract::execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Token {
                token_msg: user_2_increase_balance,
            },
        );
        assert!(res.is_ok());
        let holder_state = load_holder(&deps.storage, &user_2_address).unwrap();
        assert_eq!(deposit_2_amount, holder_state.balance);
        assert_eq!(Decimal::zero(), holder_state.index);
        assert_eq!(Decimal::zero(), holder_state.pending_rewards);
    }

    {
        //rewards incoming
        deps.querier.with_token_balances(&[(
            &psi_token_addr,
            &[(
                &MOCK_CONTRACT_ADDR.to_string(),
                &(rewards_before_receive_nasset + rewards_after_receive_nasset),
            )],
        )]);
        let update_index_msg = yield_optimizer::nasset_token_rewards::AnyoneMsg::UpdateGlobalIndex;
        let info = mock_info(&user_1_address.to_string(), &vec![]);
        let res = crate::contract::execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Anyone {
                anyone_msg: update_index_msg,
            },
        );
        assert!(res.is_ok());
    }

    {
        // -= FIRST USER SEND CLAIM message =-
        let claim_msg =
            yield_optimizer::nasset_token_rewards::AnyoneMsg::ClaimRewards { recipient: None };
        let info = mock_info(&user_1_address.to_string(), &vec![]);
        let res = crate::contract::execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Anyone {
                anyone_msg: claim_msg,
            },
        )
        .unwrap();

        assert_eq!(
            res.messages,
            vec![CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: psi_token_addr.clone(),
                send: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: user_1_address.to_string(),
                    amount: Uint128(1500), // total_rewards = 6k * share (1/4)
                })
                .unwrap(),
            })]
        );
        assert!(res.submessages.is_empty());

        let holder = load_holder(&deps.storage, &user_1_address).unwrap();
        assert_eq!(deposit_1_amount, holder.balance);
        assert_eq!(Decimal::from_str("15").unwrap(), holder.index);
        assert_eq!(Decimal::zero(), holder.pending_rewards);

        let state = load_state(&deps.storage).unwrap();
        assert_eq!(Decimal::from_str("15").unwrap(), state.global_index);
        assert_eq!(deposit_1_amount + deposit_2_amount, state.total_balance);
        assert_eq!(Uint128(4500), state.prev_reward_balance);
    }

    {
        // -= SECOND USER SEND CLAIM message =-
        let claim_msg =
            yield_optimizer::nasset_token_rewards::AnyoneMsg::ClaimRewards { recipient: None };
        let info = mock_info(&user_2_address.to_string(), &vec![]);
        let res = crate::contract::execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Anyone {
                anyone_msg: claim_msg,
            },
        )
        .unwrap();
        assert_eq!(
            res.messages,
            vec![CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: psi_token_addr.clone(),
                send: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: user_2_address.to_string(),
                    amount: Uint128(4500), // total_rewards = 6k * share (3/4)
                })
                .unwrap(),
            })]
        );
        assert!(res.submessages.is_empty());

        let holder = load_holder(&deps.storage, &user_2_address).unwrap();
        assert_eq!(deposit_2_amount, holder.balance);
        assert_eq!(Decimal::from_str("15").unwrap(), holder.index);
        assert_eq!(Decimal::zero(), holder.pending_rewards);

        let state = load_state(&deps.storage).unwrap();
        assert_eq!(Decimal::from_str("15").unwrap(), state.global_index);
        assert_eq!(deposit_1_amount + deposit_2_amount, state.total_balance);
        assert_eq!(Uint128::zero(), state.prev_reward_balance);
    }
}

#[test]
fn two_users_hold_partially_unhold_and_hold_again() {
    let psi_token_addr = "addr0001".to_string();
    let nasset_token_addr = "addr0002".to_string();
    let governance_contract_addr = "addr0003".to_string();

    let msg = yield_optimizer::nasset_token_rewards::InstantiateMsg {
        psi_token_addr: psi_token_addr.clone(),
        nasset_token_addr: nasset_token_addr.clone(),
        governance_contract_addr,
    };

    let mut deps = mock_dependencies(&[]);

    // -= INITIALIZATION =-
    {
        let env = mock_env();
        let info = mock_info("addr0010", &[]);
        crate::contract::instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();
    }

    //first user comes, increase his balance
    let user_1_address = Addr::unchecked("addr9999".to_string());
    let deposit_1_amount: Uint128 = 100u128.into();
    let user_2_address = Addr::unchecked("addr6666".to_string());
    let deposit_2_amount: Uint128 = 300u128.into();
    {
        // -= nAsset token send message to increase first user balance =-
        let user_1_increase_balance =
            yield_optimizer::nasset_token_rewards::TokenMsg::IncreaseBalance {
                address: user_1_address.to_string(),
                amount: deposit_1_amount,
            };
        let info = mock_info(&nasset_token_addr, &vec![]);
        let res = crate::contract::execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Token {
                token_msg: user_1_increase_balance,
            },
        );
        assert!(res.is_ok());
        let holder_state = load_holder(&deps.storage, &user_1_address).unwrap();
        assert_eq!(deposit_1_amount, holder_state.balance);
        assert_eq!(Decimal::zero(), holder_state.index);
        assert_eq!(Decimal::zero(), holder_state.pending_rewards);
    }

    //second user comes, increase his balance
    {
        // -= nAsset token send message to increase second user balance =-
        let user_2_increase_balance =
            yield_optimizer::nasset_token_rewards::TokenMsg::IncreaseBalance {
                address: user_2_address.to_string(),
                amount: deposit_2_amount,
            };
        let info = mock_info(&nasset_token_addr, &vec![]);
        let res = crate::contract::execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Token {
                token_msg: user_2_increase_balance,
            },
        );
        assert!(res.is_ok());
        let holder_state = load_holder(&deps.storage, &user_2_address).unwrap();
        assert_eq!(deposit_2_amount, holder_state.balance);
        assert_eq!(Decimal::zero(), holder_state.index);
        assert_eq!(Decimal::zero(), holder_state.pending_rewards);
    }

    let new_rewards_amount: Uint128 = 1000u64.into();
    {
        //rewards incoming
        deps.querier.with_token_balances(&[(
            &psi_token_addr,
            &[(&MOCK_CONTRACT_ADDR.to_string(), &new_rewards_amount)],
        )]);
        let update_index_msg = yield_optimizer::nasset_token_rewards::AnyoneMsg::UpdateGlobalIndex;
        let info = mock_info(&user_1_address.to_string(), &vec![]);
        let res = crate::contract::execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Anyone {
                anyone_msg: update_index_msg,
            },
        );
        assert!(res.is_ok());
    }

    let withdraw_1_amount: Uint128 = 30u128.into();
    let withdraw_2_amount: Uint128 = 150u128.into();
    //1/4(total stake share) * 1000 (dep1+ dep2)
    let rewards_1_amount = Uint128::from(250u64);
    {
        // -= FIRST USER SEND his nAsset =-
        let decrease_balance_msg =
            yield_optimizer::nasset_token_rewards::TokenMsg::DecreaseBalance {
                address: user_1_address.to_string(),
                amount: withdraw_1_amount,
            };
        let info = mock_info(&nasset_token_addr, &vec![]);
        let res = crate::contract::execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Token {
                token_msg: decrease_balance_msg,
            },
        );

        assert!(res.is_ok());

        let holder_state = load_holder(&deps.storage, &user_1_address).unwrap();
        assert_eq!(
            deposit_1_amount.checked_sub(withdraw_1_amount).unwrap(),
            holder_state.balance
        );
        assert_eq!(Decimal::from_str("2.5").unwrap(), holder_state.index);
        //it is 'rewards_1_amount'
        assert_eq!(
            Decimal::from_str("250").unwrap(),
            holder_state.pending_rewards
        );

        let state = load_state(&deps.storage).unwrap();
        assert_eq!(Decimal::from_str("2.5").unwrap(), state.global_index);
        assert_eq!(
            (deposit_1_amount + deposit_2_amount)
                .checked_sub(withdraw_1_amount)
                .unwrap(),
            state.total_balance
        );
        assert_eq!(new_rewards_amount, state.prev_reward_balance);

        // -= FIRST USER CLAIM rewards =-
        let claim_msg =
            yield_optimizer::nasset_token_rewards::AnyoneMsg::ClaimRewards { recipient: None };
        let info = mock_info(&user_1_address.to_string(), &vec![]);
        let res = crate::contract::execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Anyone {
                anyone_msg: claim_msg,
            },
        )
        .unwrap();

        assert_eq!(
            res.messages,
            vec![CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: psi_token_addr.clone(),
                send: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: user_1_address.to_string(),
                    amount: rewards_1_amount,
                })
                .unwrap(),
            })]
        );
        assert!(res.submessages.is_empty());

        let holder = load_holder(&deps.storage, &user_1_address).unwrap();
        assert_eq!(Decimal::zero(), holder.pending_rewards);

        let state = load_state(&deps.storage).unwrap();
        assert_eq!(
            new_rewards_amount.checked_sub(rewards_1_amount).unwrap(),
            state.prev_reward_balance
        );
    }

    // subtract rewards sent to user_1
    {
        deps.querier.with_token_balances(&[(
            &psi_token_addr,
            &[(
                &MOCK_CONTRACT_ADDR.to_string(),
                &(new_rewards_amount.checked_sub(rewards_1_amount).unwrap()),
            )],
        )]);
    }

    {
        // -= SECOND USER SEND his nAsset =-
        let rewards_2_amount = Uint128::from(750u64);
        let decrease_balance_msg =
            yield_optimizer::nasset_token_rewards::TokenMsg::DecreaseBalance {
                address: user_2_address.to_string(),
                amount: withdraw_2_amount,
            };
        let info = mock_info(&nasset_token_addr, &vec![]);
        let res = crate::contract::execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Token {
                token_msg: decrease_balance_msg,
            },
        );
        assert!(res.is_ok());

        let holder_state = load_holder(&deps.storage, &user_2_address).unwrap();
        assert_eq!(
            deposit_2_amount.checked_sub(withdraw_2_amount).unwrap(),
            holder_state.balance
        );
        assert_eq!(Decimal::from_str("2.5").unwrap(), holder_state.index);
        //it is 'rewards_2_amount'
        assert_eq!(
            Decimal::from_str("750").unwrap(),
            holder_state.pending_rewards
        );

        let state = load_state(&deps.storage).unwrap();
        assert_eq!(Decimal::from_str("2.5").unwrap(), state.global_index);
        assert_eq!(
            (deposit_1_amount + deposit_2_amount)
                .checked_sub(withdraw_1_amount)
                .unwrap()
                .checked_sub(withdraw_2_amount)
                .unwrap(),
            state.total_balance
        );
        assert_eq!(
            new_rewards_amount.checked_sub(rewards_1_amount).unwrap(),
            state.prev_reward_balance
        );

        // -= SECOND USER CLAIM rewards =-
        let claim_msg =
            yield_optimizer::nasset_token_rewards::AnyoneMsg::ClaimRewards { recipient: None };
        let info = mock_info(&user_2_address.to_string(), &vec![]);
        let res = crate::contract::execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Anyone {
                anyone_msg: claim_msg,
            },
        )
        .unwrap();

        assert_eq!(
            res.messages,
            vec![CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: psi_token_addr.clone(),
                send: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: user_2_address.to_string(),
                    amount: rewards_2_amount,
                })
                .unwrap(),
            })]
        );
        assert!(res.submessages.is_empty());

        let holder = load_holder(&deps.storage, &user_2_address).unwrap();
        assert_eq!(Decimal::zero(), holder.pending_rewards);

        let state = load_state(&deps.storage).unwrap();
        assert_eq!(Uint128::zero(), state.prev_reward_balance);
    }

    // subtract rewards sent to user_1 and user_2
    {
        deps.querier.with_token_balances(&[(
            &psi_token_addr,
            &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::zero())],
        )]);
    }

    // -= USER 1 SECOND TIME RECEIVE nAsset token =-
    {
        let second_deposit_1_amount = Uint128::from(130u64);
        let user_1_increase_balance =
            yield_optimizer::nasset_token_rewards::TokenMsg::IncreaseBalance {
                address: user_1_address.to_string(),
                amount: second_deposit_1_amount,
            };
        let info = mock_info(&nasset_token_addr, &vec![]);
        let res = crate::contract::execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Token {
                token_msg: user_1_increase_balance,
            },
        );

        assert!(res.is_ok());
        let holder_state = load_holder(&deps.storage, &user_1_address).unwrap();
        assert_eq!(
            (deposit_1_amount + second_deposit_1_amount)
                .checked_sub(withdraw_1_amount)
                .unwrap(),
            holder_state.balance
        );
        assert_eq!(Decimal::from_str("2.5").unwrap(), holder_state.index);
        assert_eq!(Decimal::zero(), holder_state.pending_rewards);

        let state = load_state(&deps.storage).unwrap();
        assert_eq!(Decimal::from_str("2.5").unwrap(), state.global_index);
        assert_eq!(
            (deposit_1_amount + deposit_2_amount + second_deposit_1_amount)
                .checked_sub(withdraw_1_amount)
                .unwrap()
                .checked_sub(withdraw_2_amount)
                .unwrap(),
            state.total_balance
        );
        assert_eq!(Uint128::zero(), state.prev_reward_balance);
    }
}
