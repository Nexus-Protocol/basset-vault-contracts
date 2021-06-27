use crate::state::{load_staker_state, load_state, StakerState, State};
use crate::tests::mock_dependencies;
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR},
    Addr, CosmosMsg, WasmMsg,
};
use cosmwasm_std::{to_binary, Uint128};
use cw20::Cw20ReceiveMsg;
use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;
use std::str::FromStr;
use yield_optimizer::nasset_staker::{AnyoneMsg, Cw20HookMsg, ExecuteMsg};

#[test]
fn first_stake() {
    let nasset_token = "addr0001".to_string();
    let psi_token = "addr0002".to_string();
    let governance_contract = "addr0003".to_string();

    let msg = yield_optimizer::nasset_staker::InstantiateMsg {
        nasset_token: nasset_token.clone(),
        psi_token: psi_token.clone(),
        governance_contract,
    };

    let mut deps = mock_dependencies(&[]);

    // -= INITIALIZATION =-
    {
        let env = mock_env();
        let info = mock_info("addr0010", &[]);
        let _ = crate::contract::instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();
    }

    //first farmer come
    let user_1_address = Addr::unchecked("addr9999".to_string());
    //it is staked_nasset_amount
    let deposit_1_amount: Uint128 = 100u128.into();
    {
        deps.querier.with_token_balances(&[(
            &nasset_token,
            &[(&MOCK_CONTRACT_ADDR.to_string(), &deposit_1_amount)],
        )]);

        // -= USER SEND nAsset tokens to nasset_staker =-
        {
            let cw20_deposit_msg = Cw20ReceiveMsg {
                sender: user_1_address.to_string(),
                amount: deposit_1_amount,
                msg: to_binary(&Cw20HookMsg::Stake).unwrap(),
            };

            let info = mock_info(&nasset_token, &vec![]);
            let res = crate::contract::execute(
                deps.as_mut(),
                mock_env(),
                info,
                ExecuteMsg::Receive(cw20_deposit_msg),
            )
            .unwrap();

            assert_eq!(res.messages, vec![]);
            assert_eq!(res.submessages, vec![]);

            let state: State = load_state(&deps.storage).unwrap();
            assert_eq!(
                State {
                    global_reward_index: Decimal256::zero(),
                    last_reward_amount: Uint256::zero(),
                    total_staked_amount: deposit_1_amount.into(),
                },
                state
            );

            let staker_state: StakerState =
                load_staker_state(&deps.storage, &user_1_address).unwrap();
            assert_eq!(
                StakerState {
                    staked_amount: Uint256::from(deposit_1_amount),
                    reward_index: Decimal256::zero(),
                    pending_rewards: Decimal256::zero(),
                },
                staker_state
            );
        }
    }
}

#[test]
fn stake_and_unstake() {
    let nasset_token = "addr0001".to_string();
    let psi_token = "addr0002".to_string();
    let governance_contract = "addr0003".to_string();

    let msg = yield_optimizer::nasset_staker::InstantiateMsg {
        nasset_token: nasset_token.clone(),
        psi_token: psi_token.clone(),
        governance_contract,
    };

    let mut deps = mock_dependencies(&[]);
    // -= INITIALIZATION =-
    {
        let env = mock_env();
        let info = mock_info("addr0010", &[]);
        let _ = crate::contract::instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();
    }

    //first farmer come
    let user_1_address = Addr::unchecked("addr9999".to_string());
    //it is staked_nasset_amount
    let deposit_1_amount: Uint128 = 100u128.into();
    {
        deps.querier.with_token_balances(&[(
            &nasset_token,
            &[(&MOCK_CONTRACT_ADDR.to_string(), &deposit_1_amount)],
        )]);

        // -= USER SEND nAsset tokens to nasset_staker =-
        {
            let cw20_deposit_msg = Cw20ReceiveMsg {
                sender: user_1_address.to_string(),
                amount: deposit_1_amount,
                msg: to_binary(&Cw20HookMsg::Stake).unwrap(),
            };

            let info = mock_info(&nasset_token, &vec![]);
            crate::contract::execute(
                deps.as_mut(),
                mock_env(),
                info,
                ExecuteMsg::Receive(cw20_deposit_msg),
            )
            .unwrap();
        }
    }

    // -= USER SEND unstake nasset tokens =-
    {
        let unstake_msg = ExecuteMsg::Anyone {
            anyone_msg: AnyoneMsg::Unstake {
                amount: Uint256::from(deposit_1_amount),
                to: None,
            },
        };

        let info = mock_info(&user_1_address.to_string(), &vec![]);
        let res = crate::contract::execute(deps.as_mut(), mock_env(), info, unstake_msg).unwrap();

        assert_eq!(
            res.messages,
            vec![CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: nasset_token.clone(),
                send: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: user_1_address.to_string(),
                    amount: deposit_1_amount,
                })
                .unwrap(),
            })]
        );
        assert_eq!(res.submessages, vec![]);

        let state: State = load_state(&deps.storage).unwrap();
        assert_eq!(
            State {
                global_reward_index: Decimal256::zero(),
                last_reward_amount: Uint256::zero(),
                total_staked_amount: Uint256::zero(),
            },
            state
        );

        let staker_state: StakerState = load_staker_state(&deps.storage, &user_1_address).unwrap();
        assert_eq!(
            StakerState {
                staked_amount: Uint256::zero(),
                reward_index: Decimal256::zero(),
                pending_rewards: Decimal256::zero(),
            },
            staker_state
        );
    }
}

#[test]
fn stake_and_unstake_partially() {
    let nasset_token = "addr0001".to_string();
    let psi_token = "addr0002".to_string();
    let governance_contract = "addr0003".to_string();

    let msg = yield_optimizer::nasset_staker::InstantiateMsg {
        nasset_token: nasset_token.clone(),
        psi_token: psi_token.clone(),
        governance_contract,
    };

    let mut deps = mock_dependencies(&[]);

    // -= INITIALIZATION =-
    {
        let env = mock_env();
        let info = mock_info("addr0010", &[]);
        let _ = crate::contract::instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();
    }

    //first farmer come
    let user_1_address = Addr::unchecked("addr9999".to_string());
    //it is staked_casset_amount
    let deposit_1_amount: Uint128 = 100u128.into();
    let withdraw_amount: Uint128 = 30u128.into();
    {
        deps.querier.with_token_balances(&[(
            &nasset_token,
            &[(&MOCK_CONTRACT_ADDR.to_string(), &deposit_1_amount)],
        )]);

        // -= USER SEND nAsset tokens to nasset_staker =-
        {
            let cw20_deposit_msg = Cw20ReceiveMsg {
                sender: user_1_address.to_string(),
                amount: deposit_1_amount,
                msg: to_binary(&Cw20HookMsg::Stake).unwrap(),
            };

            let info = mock_info(&nasset_token, &vec![]);
            crate::contract::execute(
                deps.as_mut(),
                mock_env(),
                info,
                ExecuteMsg::Receive(cw20_deposit_msg),
            )
            .unwrap();
        }
    }

    // -= USER SEND unstake nasset tokens =-
    {
        let unstake_msg = ExecuteMsg::Anyone {
            anyone_msg: AnyoneMsg::Unstake {
                amount: Uint256::from(withdraw_amount),
                to: None,
            },
        };

        let info = mock_info(&user_1_address.to_string(), &vec![]);
        crate::contract::execute(deps.as_mut(), mock_env(), info, unstake_msg).unwrap();

        let state: State = load_state(&deps.storage).unwrap();
        assert_eq!(
            State {
                global_reward_index: Decimal256::zero(),
                last_reward_amount: Uint256::zero(),
                total_staked_amount: deposit_1_amount
                    .checked_sub(withdraw_amount)
                    .unwrap()
                    .into(),
            },
            state
        );

        let staker_state: StakerState = load_staker_state(&deps.storage, &user_1_address).unwrap();
        assert_eq!(
            StakerState {
                staked_amount: Uint256::from(
                    deposit_1_amount.checked_sub(withdraw_amount).unwrap()
                ),
                reward_index: Decimal256::zero(),
                pending_rewards: Decimal256::zero(),
            },
            staker_state
        );
    }
}

#[test]
fn two_users_stake_partially_unstake_stake_again() {
    let nasset_token = "addr0001".to_string();
    let psi_token = "addr0002".to_string();
    let governance_contract = "addr0003".to_string();

    let msg = yield_optimizer::nasset_staker::InstantiateMsg {
        nasset_token: nasset_token.clone(),
        psi_token: psi_token.clone(),
        governance_contract,
    };

    let mut deps = mock_dependencies(&[]);

    // -= INITIALIZATION =-
    {
        let env = mock_env();
        let info = mock_info("addr0010", &[]);
        let _ = crate::contract::instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();
    }

    let user_1_address = Addr::unchecked("addr9999".to_string());
    let deposit_1_amount: Uint128 = 100u128.into();
    let user_2_address = Addr::unchecked("addr6666".to_string());
    let deposit_2_amount: Uint128 = 300u128.into();
    {
        // -= USER 1 SEND nAsset tokens to nasset_staker =-
        {
            deps.querier.with_token_balances(&[(
                &nasset_token,
                &[(&MOCK_CONTRACT_ADDR.to_string(), &deposit_1_amount)],
            )]);

            let cw20_deposit_msg = Cw20ReceiveMsg {
                sender: user_1_address.to_string(),
                amount: deposit_1_amount,
                msg: to_binary(&Cw20HookMsg::Stake).unwrap(),
            };

            let info = mock_info(&nasset_token, &vec![]);
            let mut env = mock_env();
            env.block.height = 68882;
            crate::contract::execute(
                deps.as_mut(),
                env,
                info,
                ExecuteMsg::Receive(cw20_deposit_msg),
            )
            .unwrap();
        }

        // -= USER 2 SEND nAsset tokens to nasset_staker =-
        {
            deps.querier.with_token_balances(&[(
                &nasset_token,
                &[(
                    &MOCK_CONTRACT_ADDR.to_string(),
                    &(deposit_1_amount + deposit_2_amount),
                )],
            )]);

            let cw20_deposit_msg = Cw20ReceiveMsg {
                sender: user_2_address.to_string(),
                amount: deposit_2_amount,
                msg: to_binary(&Cw20HookMsg::Stake).unwrap(),
            };

            let info = mock_info(&nasset_token, &vec![]);
            crate::contract::execute(
                deps.as_mut(),
                mock_env(),
                info,
                ExecuteMsg::Receive(cw20_deposit_msg),
            )
            .unwrap();

            let state: State = load_state(&deps.storage).unwrap();
            assert_eq!(
                State {
                    global_reward_index: Decimal256::zero(),
                    last_reward_amount: Uint256::zero(),
                    total_staked_amount: (deposit_2_amount + deposit_1_amount).into(),
                },
                state
            );

            let staker_state: StakerState =
                load_staker_state(&deps.storage, &user_2_address).unwrap();
            assert_eq!(
                StakerState {
                    staked_amount: Uint256::from(deposit_2_amount),
                    reward_index: Decimal256::zero(),
                    pending_rewards: Decimal256::zero(),
                },
                staker_state
            );
        }
    }

    let new_rewards_amount = Uint128::from(1_000u64);
    // -= SOME REWARDS COME (nasset balance increased) =-
    {
        deps.querier.with_token_balances(&[
            (
                &nasset_token,
                &[(
                    &MOCK_CONTRACT_ADDR.to_string(),
                    &(deposit_1_amount + deposit_2_amount),
                )],
            ),
            (
                &psi_token,
                &[(&MOCK_CONTRACT_ADDR.to_string(), &new_rewards_amount)],
            ),
        ]);
    }

    let withdraw_1_amount: Uint128 = 30u128.into();
    let withdraw_2_amount: Uint128 = 150u128.into();
    //1/4(total stake share) * 1000 (dep1+ dep2)
    let rewards_1_amount = Uint128::from(250u64);
    // -= USER 1 partially unstake =-
    {
        let unstake_msg = ExecuteMsg::Anyone {
            anyone_msg: AnyoneMsg::Unstake {
                amount: Uint256::from(withdraw_1_amount),
                to: None,
            },
        };

        let info = mock_info(&user_1_address.to_string(), &vec![]);
        let res = crate::contract::execute(deps.as_mut(), mock_env(), info, unstake_msg).unwrap();

        let state: State = load_state(&deps.storage).unwrap();
        assert_eq!(
            State {
                global_reward_index: Decimal256::from_str("2.5").unwrap(), //1000/400
                last_reward_amount: new_rewards_amount
                    .checked_sub(rewards_1_amount)
                    .unwrap()
                    .into(),
                total_staked_amount: (deposit_1_amount + deposit_2_amount)
                    .checked_sub(withdraw_1_amount)
                    .unwrap()
                    .into(),
            },
            state
        );

        let staker_state: StakerState = load_staker_state(&deps.storage, &user_1_address).unwrap();
        assert_eq!(
            StakerState {
                staked_amount: Uint256::from(
                    deposit_1_amount.checked_sub(withdraw_1_amount).unwrap()
                ),
                reward_index: Decimal256::from_str("2.5").unwrap(),
                pending_rewards: Decimal256::zero(),
            },
            staker_state
        );

        assert_eq!(
            res.messages,
            vec![
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: nasset_token.clone(),
                    send: vec![],
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: user_1_address.to_string(),
                        amount: withdraw_1_amount,
                    })
                    .unwrap(),
                }),
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: psi_token.clone(),
                    send: vec![],
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: user_1_address.to_string(),
                        amount: rewards_1_amount,
                    })
                    .unwrap(),
                })
            ]
        );
        assert_eq!(res.submessages, vec![]);
    }

    // subtract rewards sent to user_1
    // subtract user_1 unstaked casset
    {
        deps.querier.with_token_balances(&[
            (
                &nasset_token,
                &[(
                    &MOCK_CONTRACT_ADDR.to_string(),
                    &((deposit_1_amount + deposit_2_amount)
                        .checked_sub(withdraw_1_amount)
                        .unwrap()),
                )],
            ),
            (
                &psi_token,
                &[(
                    &MOCK_CONTRACT_ADDR.to_string(),
                    &(new_rewards_amount.checked_sub(rewards_1_amount).unwrap()),
                )],
            ),
        ]);
    }

    // -= USER 2 partially unstake =-
    {
        let rewards_2_amount = Uint128::from(750u64);
        let unstake_msg = ExecuteMsg::Anyone {
            anyone_msg: AnyoneMsg::Unstake {
                amount: Uint256::from(withdraw_2_amount),
                to: None,
            },
        };

        let info = mock_info(&user_2_address.to_string(), &vec![]);
        let res = crate::contract::execute(deps.as_mut(), mock_env(), info, unstake_msg).unwrap();

        let state: State = load_state(&deps.storage).unwrap();
        assert_eq!(
            State {
                global_reward_index: Decimal256::from_str("2.5").unwrap(),
                last_reward_amount: Uint256::zero(),
                total_staked_amount: (deposit_1_amount + deposit_2_amount)
                    .checked_sub(withdraw_1_amount)
                    .unwrap()
                    .checked_sub(withdraw_2_amount)
                    .unwrap()
                    .into()
            },
            state
        );

        let staker_state: StakerState = load_staker_state(&deps.storage, &user_2_address).unwrap();
        assert_eq!(
            StakerState {
                staked_amount: Uint256::from(
                    deposit_2_amount.checked_sub(withdraw_2_amount).unwrap()
                ),
                reward_index: Decimal256::from_str("2.5").unwrap(),
                pending_rewards: Decimal256::zero(),
            },
            staker_state
        );

        assert_eq!(
            res.messages,
            vec![
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: nasset_token.clone(),
                    send: vec![],
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: user_2_address.to_string(),
                        amount: withdraw_2_amount,
                    })
                    .unwrap(),
                }),
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: psi_token.clone(),
                    send: vec![],
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: user_2_address.to_string(),
                        amount: rewards_2_amount,
                    })
                    .unwrap(),
                })
            ]
        );
        assert_eq!(res.submessages, vec![]);
    }

    // -= USER 1 SECOND TIME SEND nAsset tokens to nasset_staker =-
    {
        let second_deposit_1_amount = Uint128::from(130u64);
        deps.querier.with_token_balances(&[(
            &nasset_token,
            &[(
                &MOCK_CONTRACT_ADDR.to_string(),
                &((deposit_1_amount + deposit_2_amount + second_deposit_1_amount)
                    .checked_sub(withdraw_1_amount)
                    .unwrap()
                    .checked_sub(withdraw_2_amount)
                    .unwrap()),
            )],
        )]);

        let cw20_deposit_msg = Cw20ReceiveMsg {
            sender: user_1_address.to_string(),
            amount: second_deposit_1_amount,
            msg: to_binary(&Cw20HookMsg::Stake).unwrap(),
        };

        let info = mock_info(&nasset_token, &vec![]);
        crate::contract::execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Receive(cw20_deposit_msg),
        )
        .unwrap();

        let state: State = load_state(&deps.storage).unwrap();
        assert_eq!(
            State {
                global_reward_index: Decimal256::from_str("2.5").unwrap(),
                last_reward_amount: Uint256::zero(),
                total_staked_amount: (deposit_1_amount
                    + deposit_2_amount
                    + second_deposit_1_amount)
                    .checked_sub(withdraw_1_amount)
                    .unwrap()
                    .checked_sub(withdraw_2_amount)
                    .unwrap()
                    .into(),
            },
            state
        );

        let staker_state: StakerState = load_staker_state(&deps.storage, &user_1_address).unwrap();
        assert_eq!(
            StakerState {
                staked_amount: (deposit_1_amount + second_deposit_1_amount)
                    .checked_sub(withdraw_1_amount)
                    .unwrap()
                    .into(),
                reward_index: Decimal256::from_str("2.5").unwrap(),
                pending_rewards: Decimal256::zero(),
            },
            staker_state
        );
    }
}

#[test]
fn first_staker_come_but_rewards_already_there() {
    let nasset_token = "addr0001".to_string();
    let psi_token = "addr0002".to_string();
    let governance_contract = "addr0003".to_string();

    let msg = yield_optimizer::nasset_staker::InstantiateMsg {
        nasset_token: nasset_token.clone(),
        psi_token: psi_token.clone(),
        governance_contract,
    };

    let mut deps = mock_dependencies(&[]);

    // -= INITIALIZATION =-
    {
        let env = mock_env();
        let info = mock_info("addr0010", &[]);
        let _ = crate::contract::instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();
    }

    //first farmer come
    let user_1_address = Addr::unchecked("addr9999".to_string());
    //it is staked_nasset_amount
    let deposit_1_amount: Uint128 = 100u128.into();
    //rewards already there
    let rewards: Uint128 = 1000u64.into();
    {
        deps.querier.with_token_balances(&[
            (
                &nasset_token,
                &[(&MOCK_CONTRACT_ADDR.to_string(), &deposit_1_amount)],
            ),
            (&psi_token, &[(&MOCK_CONTRACT_ADDR.to_string(), &rewards)]),
        ]);

        // -= USER SEND nAsset tokens to nasset_staker =-
        {
            let cw20_deposit_msg = Cw20ReceiveMsg {
                sender: user_1_address.to_string(),
                amount: deposit_1_amount,
                msg: to_binary(&Cw20HookMsg::Stake).unwrap(),
            };

            let info = mock_info(&nasset_token, &vec![]);
            let res = crate::contract::execute(
                deps.as_mut(),
                mock_env(),
                info,
                ExecuteMsg::Receive(cw20_deposit_msg),
            )
            .unwrap();

            assert_eq!(res.messages, vec![]);
            assert_eq!(res.submessages, vec![]);

            let state: State = load_state(&deps.storage).unwrap();
            assert_eq!(
                State {
                    global_reward_index: Decimal256::zero(),
                    last_reward_amount: Uint256::zero(),
                    total_staked_amount: deposit_1_amount.into(),
                },
                state
            );

            let staker_state: StakerState =
                load_staker_state(&deps.storage, &user_1_address).unwrap();
            assert_eq!(
                StakerState {
                    staked_amount: Uint256::from(deposit_1_amount),
                    reward_index: Decimal256::zero(),
                    pending_rewards: Decimal256::zero(),
                },
                staker_state
            );
        }
    }
}

#[test]
fn stake_and_unstake_after_rewards_already_there() {
    let nasset_token = "addr0001".to_string();
    let psi_token = "addr0002".to_string();
    let governance_contract = "addr0003".to_string();

    let msg = yield_optimizer::nasset_staker::InstantiateMsg {
        nasset_token: nasset_token.clone(),
        psi_token: psi_token.clone(),
        governance_contract,
    };

    let mut deps = mock_dependencies(&[]);

    // -= INITIALIZATION =-
    {
        let env = mock_env();
        let info = mock_info("addr0010", &[]);
        let _ = crate::contract::instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();
    }

    //first farmer come
    let user_1_address = Addr::unchecked("addr9999".to_string());
    //it is staked_nasset_amount
    let deposit_1_amount: Uint128 = 100u128.into();
    //rewards already there
    let rewards_before_stake: Uint128 = 1000u64.into();
    let rewards_after_stake: Uint128 = 5000u64.into();
    {
        deps.querier.with_token_balances(&[
            (
                &nasset_token,
                &[(&MOCK_CONTRACT_ADDR.to_string(), &deposit_1_amount)],
            ),
            (
                &psi_token,
                &[(&MOCK_CONTRACT_ADDR.to_string(), &rewards_before_stake)],
            ),
        ]);

        // -= USER SEND nAsset tokens to nasset_staker =-
        {
            let cw20_deposit_msg = Cw20ReceiveMsg {
                sender: user_1_address.to_string(),
                amount: deposit_1_amount,
                msg: to_binary(&Cw20HookMsg::Stake).unwrap(),
            };

            let info = mock_info(&nasset_token, &vec![]);
            let res = crate::contract::execute(
                deps.as_mut(),
                mock_env(),
                info,
                ExecuteMsg::Receive(cw20_deposit_msg),
            )
            .unwrap();

            assert_eq!(res.messages, vec![]);
            assert_eq!(res.submessages, vec![]);

            let state: State = load_state(&deps.storage).unwrap();
            assert_eq!(
                State {
                    global_reward_index: Decimal256::zero(),
                    last_reward_amount: Uint256::zero(),
                    total_staked_amount: deposit_1_amount.into(),
                },
                state
            );

            let staker_state: StakerState =
                load_staker_state(&deps.storage, &user_1_address).unwrap();
            assert_eq!(
                StakerState {
                    staked_amount: Uint256::from(deposit_1_amount),
                    reward_index: Decimal256::zero(),
                    pending_rewards: Decimal256::zero(),
                },
                staker_state
            );
        }
    }

    {
        deps.querier.with_token_balances(&[
            (
                &nasset_token,
                &[(&MOCK_CONTRACT_ADDR.to_string(), &deposit_1_amount)],
            ),
            (
                &psi_token,
                &[(
                    &MOCK_CONTRACT_ADDR.to_string(),
                    &(rewards_before_stake + rewards_after_stake),
                )],
            ),
        ]);

        // -= USER SEND UNSTAKE all =-
        {
            let unstake_msg = ExecuteMsg::Anyone {
                anyone_msg: AnyoneMsg::Unstake {
                    amount: deposit_1_amount.into(),
                    to: None,
                },
            };
            let info = mock_info(&user_1_address.to_string(), &vec![]);
            let res =
                crate::contract::execute(deps.as_mut(), mock_env(), info, unstake_msg).unwrap();

            assert_eq!(
                res.messages,
                vec![
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: nasset_token.clone(),
                        send: vec![],
                        msg: to_binary(&Cw20ExecuteMsg::Transfer {
                            recipient: user_1_address.to_string(),
                            amount: deposit_1_amount.into(),
                        })
                        .unwrap(),
                    }),
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: psi_token.clone(),
                        send: vec![],
                        msg: to_binary(&Cw20ExecuteMsg::Transfer {
                            recipient: user_1_address.to_string(),
                            amount: (rewards_after_stake + rewards_before_stake).into(),
                        })
                        .unwrap(),
                    }),
                ]
            );
            assert!(res.submessages.is_empty());

            let state: State = load_state(&deps.storage).unwrap();
            assert_eq!(
                State {
                    global_reward_index: Decimal256::from_str("60").unwrap(),
                    last_reward_amount: Uint256::zero(),
                    total_staked_amount: Uint256::zero(),
                },
                state
            );

            let staker_state: StakerState =
                load_staker_state(&deps.storage, &user_1_address).unwrap();
            assert_eq!(
                StakerState {
                    staked_amount: Uint256::zero(),
                    reward_index: Decimal256::from_str("60").unwrap(),
                    pending_rewards: Decimal256::zero(),
                },
                staker_state
            );
        }
    }
}

#[test]
fn second_staker_after_rewards_already_there() {
    let nasset_token = "addr0001".to_string();
    let psi_token = "addr0002".to_string();
    let governance_contract = "addr0003".to_string();

    let msg = yield_optimizer::nasset_staker::InstantiateMsg {
        nasset_token: nasset_token.clone(),
        psi_token: psi_token.clone(),
        governance_contract,
    };

    let mut deps = mock_dependencies(&[]);

    // -= INITIALIZATION =-
    {
        let env = mock_env();
        let info = mock_info("addr0010", &[]);
        let _ = crate::contract::instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();
    }

    let user_1_address = Addr::unchecked("addr9999".to_string());
    let deposit_1_amount: Uint128 = 100u128.into();
    let user_2_address = Addr::unchecked("addr6666".to_string());
    let deposit_2_amount: Uint128 = 300u128.into();
    //rewards already there
    let rewards_before_stake: Uint128 = 1000u64.into();
    let rewards_after_stake: Uint128 = 5000u64.into();
    {
        deps.querier.with_token_balances(&[
            (
                &nasset_token,
                &[(&MOCK_CONTRACT_ADDR.to_string(), &deposit_1_amount)],
            ),
            (
                &psi_token,
                &[(&MOCK_CONTRACT_ADDR.to_string(), &rewards_before_stake)],
            ),
        ]);

        // -= USER SEND nAsset tokens to nasset_staker =-
        {
            let cw20_deposit_msg = Cw20ReceiveMsg {
                sender: user_1_address.to_string(),
                amount: deposit_1_amount,
                msg: to_binary(&Cw20HookMsg::Stake).unwrap(),
            };

            let info = mock_info(&nasset_token, &vec![]);
            let res = crate::contract::execute(
                deps.as_mut(),
                mock_env(),
                info,
                ExecuteMsg::Receive(cw20_deposit_msg),
            )
            .unwrap();

            assert_eq!(res.messages, vec![]);
            assert_eq!(res.submessages, vec![]);

            let state: State = load_state(&deps.storage).unwrap();
            assert_eq!(
                State {
                    global_reward_index: Decimal256::zero(),
                    last_reward_amount: Uint256::zero(),
                    total_staked_amount: deposit_1_amount.into(),
                },
                state
            );

            let staker_state: StakerState =
                load_staker_state(&deps.storage, &user_1_address).unwrap();
            assert_eq!(
                StakerState {
                    staked_amount: Uint256::from(deposit_1_amount),
                    reward_index: Decimal256::zero(),
                    pending_rewards: Decimal256::zero(),
                },
                staker_state
            );
        }
    }

    {
        //new rewards comes
        deps.querier.with_token_balances(&[
            (
                &nasset_token,
                &[(
                    &MOCK_CONTRACT_ADDR.to_string(),
                    &(deposit_1_amount + deposit_2_amount),
                )],
            ),
            (
                &psi_token,
                &[(
                    &MOCK_CONTRACT_ADDR.to_string(),
                    &(rewards_before_stake + rewards_after_stake),
                )],
            ),
        ]);

        // -= USER 2 SEND nAsset tokens to nasset_staker =-
        {
            let cw20_deposit_msg = Cw20ReceiveMsg {
                sender: user_2_address.to_string(),
                amount: deposit_2_amount,
                msg: to_binary(&Cw20HookMsg::Stake).unwrap(),
            };

            let info = mock_info(&nasset_token, &vec![]);
            crate::contract::execute(
                deps.as_mut(),
                mock_env(),
                info,
                ExecuteMsg::Receive(cw20_deposit_msg),
            )
            .unwrap();

            let state: State = load_state(&deps.storage).unwrap();
            assert_eq!(
                State {
                    global_reward_index: Decimal256::from_str("60").unwrap(),
                    last_reward_amount: (rewards_after_stake + rewards_before_stake).into(),
                    total_staked_amount: (deposit_2_amount + deposit_1_amount).into(),
                },
                state
            );

            let staker_state: StakerState =
                load_staker_state(&deps.storage, &user_2_address).unwrap();
            assert_eq!(
                StakerState {
                    staked_amount: Uint256::from(deposit_2_amount),
                    reward_index: Decimal256::from_str("60").unwrap(),
                    pending_rewards: Decimal256::zero(),
                },
                staker_state
            );
        }
    }

    {
        //rewards for second user comes
        let rewards_for_second_user = Uint128::from(500u64);
        deps.querier.with_token_balances(&[
            (
                &nasset_token,
                &[(
                    &MOCK_CONTRACT_ADDR.to_string(),
                    &(deposit_1_amount + deposit_2_amount),
                )],
            ),
            (
                &psi_token,
                &[(
                    &MOCK_CONTRACT_ADDR.to_string(),
                    &(rewards_before_stake + rewards_after_stake + rewards_for_second_user),
                )],
            ),
        ]);

        // -= SECOND USER SEND UNSTAKE all =-
        {
            let unstake_msg = ExecuteMsg::Anyone {
                anyone_msg: AnyoneMsg::Unstake {
                    amount: deposit_2_amount.into(),
                    to: None,
                },
            };
            let info = mock_info(&user_2_address.to_string(), &vec![]);
            let res =
                crate::contract::execute(deps.as_mut(), mock_env(), info, unstake_msg).unwrap();

            assert_eq!(
                res.messages,
                vec![
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: nasset_token.clone(),
                        send: vec![],
                        msg: to_binary(&Cw20ExecuteMsg::Transfer {
                            recipient: user_2_address.to_string(),
                            amount: deposit_2_amount.into(),
                        })
                        .unwrap(),
                    }),
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: psi_token.clone(),
                        send: vec![],
                        msg: to_binary(&Cw20ExecuteMsg::Transfer {
                            recipient: user_2_address.to_string(),
                            //second user have 3/4 rewards share
                            //500 * 3/4 = 375
                            amount: 375u64.into(),
                        })
                        .unwrap(),
                    }),
                ]
            );
            assert!(res.submessages.is_empty());

            let state: State = load_state(&deps.storage).unwrap();
            println!("state.last_reward_amount: {}", state.last_reward_amount);
            assert_eq!(
                State {
                    // old_index += 500/400
                    global_reward_index: Decimal256::from_str("61.25").unwrap(),
                    last_reward_amount: 125u64.into(), //500 - claimed 375
                    total_staked_amount: deposit_1_amount.into(),
                },
                state
            );

            let staker_state: StakerState =
                load_staker_state(&deps.storage, &user_2_address).unwrap();
            assert_eq!(
                StakerState {
                    staked_amount: Uint256::zero(),
                    reward_index: Decimal256::from_str("61.25").unwrap(),
                    pending_rewards: Decimal256::zero(),
                },
                staker_state
            );
        }
    }
}
