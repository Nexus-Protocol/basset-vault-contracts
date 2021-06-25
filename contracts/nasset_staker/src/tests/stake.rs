use crate::state::{load_config, load_staker_state, load_state, Config, StakerState, State};
use crate::tests::mock_dependencies;
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR},
    Addr, Coin, CosmosMsg, Decimal, WasmMsg,
};
use cosmwasm_std::{to_binary, Uint128};
use cw20::Cw20ReceiveMsg;
use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;
use std::str::FromStr;
use yield_optimizer::{
    basset_farmer::ExecuteMsg as BAssetFarmerExecuteMsg,
    nasset_staker::{AnyoneMsg, Cw20HookMsg, ExecuteMsg},
    querier::{AnchorMarketEpochStateResponse, BorrowerInfoResponse},
};

#[test]
fn first_stake() {
    //numbers for reward calc get 'reward_calc_first_reward'
    let nasset_token = "addr0001".to_string();

    let msg = yield_optimizer::nasset_staker::InstantiateMsg {
        nasset_token: nasset_token.clone(),
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
                    // global_reward_index: Decimal256::from_str("245").unwrap(),
                    // last_reward_amount: Decimal256::from_str("24500").unwrap(),
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
    //numbers for reward calc get 'reward_calc_first_reward'
    let nasset_token = "addr0001".to_string();

    let msg = yield_optimizer::nasset_staker::InstantiateMsg {
        nasset_token: nasset_token.clone(),
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
                global_reward_index: Decimal256::from_str("245").unwrap(),
                last_reward_amount: Uint256::from(24_500u64),
                total_staked_amount: deposit_1_amount.into(),
            },
            state
        );

        let staker_state: StakerState = load_staker_state(&deps.storage, &user_1_address).unwrap();
        assert_eq!(
            StakerState {
                staked_amount: Uint256::zero(),
                reward_index: Decimal256::from_str("245").unwrap(),
                pending_rewards: Decimal256::zero(),
            },
            staker_state
        );
    }
}

#[test]
fn stake_and_unstake_partially() {
    //numbers for reward calc get 'reward_calc_first_reward'
    let nasset_token = "addr0001".to_string();

    let msg = yield_optimizer::nasset_staker::InstantiateMsg {
        nasset_token: nasset_token.clone(),
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

        let staker_state: StakerState = load_staker_state(&deps.storage, &user_1_address).unwrap();
        assert_eq!(
            StakerState {
                staked_amount: Uint256::from(
                    deposit_1_amount.checked_sub(withdraw_amount).unwrap()
                ),
                reward_index: Decimal256::from_str("245").unwrap(),
                pending_rewards: Decimal256::zero(),
            },
            staker_state
        );
    }
}

#[test]
fn two_users_staking() {
    //numbers for reward calc get 'reward_calc_first_reward'
    let nasset_token = "addr0001".to_string();

    let msg = yield_optimizer::nasset_staker::InstantiateMsg {
        nasset_token: nasset_token.clone(),
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
                    global_reward_index: Decimal256::from_str("245").unwrap(),
                    last_reward_amount: Uint256::from(24_500u64),
                    total_staked_amount: (deposit_2_amount + deposit_1_amount).into(),
                },
                state
            );

            let staker_state: StakerState =
                load_staker_state(&deps.storage, &user_2_address).unwrap();
            assert_eq!(
                StakerState {
                    staked_amount: Uint256::from(deposit_2_amount),
                    reward_index: Decimal256::from_str("245").unwrap(),
                    pending_rewards: Decimal256::zero(),
                },
                staker_state
            );
        }
    }

    let new_rewards_amount = Uint128::from(20_000u64);
    // -= SOME REWARDS COME (nasset balance increased) =-
    {
        deps.querier.with_token_balances(&[(
            &nasset_token,
            &[(
                &MOCK_CONTRACT_ADDR.to_string(),
                &(deposit_1_amount + deposit_2_amount + new_rewards_amount),
            )],
        )]);
    }

    let withdraw_1_amount: Uint128 = 30u128.into();
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
                global_reward_index: Decimal256::from_str("506.25").unwrap(),
                last_reward_amount: Uint256::from(102_875u64),
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
                reward_index: Decimal256::from_str("506.25").unwrap(),
                pending_rewards: Decimal256::zero(),
            },
            staker_state
        );

        assert_eq!(
            res.messages,
            vec![CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: nasset_token.clone(),
                send: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: user_1_address.to_string(),
                    amount: withdraw_1_amount,
                })
                .unwrap(),
            })]
        );
        assert_eq!(res.submessages, vec![]);
    }

    // subtract rewards sent to user_1
    // subtract user_1 unstaked casset
    {
        deps.querier.with_token_balances(&[(
            &nasset_token,
            &[(
                &MOCK_CONTRACT_ADDR.to_string(),
                &((deposit_1_amount + deposit_2_amount)
                    .checked_sub(withdraw_1_amount)
                    .unwrap()),
            )],
        )]);
    }

    // -= USER 2 partially unstake =-
    {
        let withdraw_2_amount: Uint128 = 150u128.into();
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
                global_reward_index: Decimal256::from_str("506.25").unwrap(),
                last_reward_amount: Uint256::from(24_500u64),
                total_staked_amount: (deposit_2_amount + deposit_1_amount)
                    .checked_sub(withdraw_1_amount)
                    .unwrap()
                    .checked_sub(withdraw_2_amount)
                    .unwrap()
                    .into(),
            },
            state
        );

        let staker_state: StakerState = load_staker_state(&deps.storage, &user_2_address).unwrap();
        assert_eq!(
            StakerState {
                staked_amount: Uint256::from(
                    deposit_2_amount.checked_sub(withdraw_2_amount).unwrap()
                ),
                reward_index: Decimal256::from_str("506.25").unwrap(),
                pending_rewards: Decimal256::zero(),
            },
            staker_state
        );

        assert_eq!(
            res.messages,
            vec![CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: nasset_token.clone(),
                send: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: user_2_address.to_string(),
                    amount: withdraw_2_amount,
                })
                .unwrap(),
            })]
        );
        assert_eq!(res.submessages, vec![]);
    }
}

//TODO: user 1 stake, some rewards come, user 2 stake
//TODO: basset_farmer balance changed in negative way
//TODO: claim rewards test (in new file)
//TODO: first user comes, but there is already some rewards! Should we gave them to him, or what?
