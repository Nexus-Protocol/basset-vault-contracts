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
    basset_farmer::{CAssetStakerMsg, ExecuteMsg as BAssetFarmerExecuteMsg},
    casset_staking::{AnyoneMsg, Cw20HookMsg, ExecuteMsg},
    querier::{AnchorMarketEpochStateResponse, BorrowerInfoResponse},
};

#[test]
fn first_stake() {
    //numbers for reward calc get 'reward_calc_first_reward'
    let casset_token = "addr0001".to_string();
    let aterra_token = "addr0002".to_string();
    let stable_denom = "uust".to_string();
    let basset_farmer_contract = "addr0003".to_string();
    let anchor_market_contract = "addr0004".to_string();

    let msg = yield_optimizer::casset_staking::InstantiateMsg {
        casset_token: casset_token.clone(),
        aterra_token: aterra_token.clone(),
        stable_denom: stable_denom.clone(),
        basset_farmer_contract: basset_farmer_contract.clone(),
        anchor_market_contract: anchor_market_contract.clone(),
    };

    let basset_farmer_loan_amount = Uint256::from(200_000u64);
    let aterra_exchange_rate = Decimal256::from_str("1.1").unwrap();
    let aterra_amount = Uint256::from(195_000u64);
    let stable_coin_balance = Uint128::from(10_000u64);

    let mut deps = mock_dependencies(&[]);
    //to: 4. get amount of UST
    deps.querier.update_base_balance(
        &basset_farmer_contract,
        vec![Coin {
            denom: stable_denom.to_string(),
            amount: stable_coin_balance,
        }],
    );

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
        deps.querier.with_token_balances(&[
            (
                &casset_token,
                &[(&MOCK_CONTRACT_ADDR.to_string(), &deposit_1_amount)],
            ),
            //to: 2. get amount of aUST
            (
                &aterra_token,
                &[(&basset_farmer_contract.clone(), &aterra_amount.into())],
            ),
        ]);
        //to: 3. get aUST to UST ratio
        deps.querier.with_wasm_query_response(&[(
            &anchor_market_contract,
            &to_binary(&AnchorMarketEpochStateResponse {
                exchange_rate: aterra_exchange_rate,
                aterra_supply: Uint256::from(1_000_000u64),
            })
            .unwrap(),
        )]);
        //to: 1. get amount of borrowed UST
        deps.querier.with_loan(&[(
            &anchor_market_contract,
            &[(
                &basset_farmer_contract.to_string(),
                &BorrowerInfoResponse {
                    borrower: basset_farmer_contract.to_string(),
                    interest_index: Decimal256::one(),
                    reward_index: Decimal256::zero(),
                    loan_amount: basset_farmer_loan_amount,
                    pending_rewards: Decimal256::zero(),
                },
            )],
        )]);

        // -= USER SEND cAsset tokens to casset_staker =-
        {
            let cw20_deposit_msg = Cw20ReceiveMsg {
                sender: user_1_address.to_string(),
                amount: deposit_1_amount,
                msg: to_binary(&Cw20HookMsg::Stake).unwrap(),
            };

            let info = mock_info(&casset_token, &vec![]);
            let current_block_height = 68882;
            let mut env = mock_env();
            env.block.height = current_block_height;
            let res = crate::contract::execute(
                deps.as_mut(),
                env,
                info,
                ExecuteMsg::Receive(cw20_deposit_msg),
            )
            .unwrap();

            assert_eq!(res.messages, vec![]);
            assert_eq!(res.submessages, vec![]);

            let state: State = load_state(&deps.storage).unwrap();
            assert_eq!(
                State {
                    global_reward_index: Decimal256::from_str("245").unwrap(),
                    last_reward_amount: Decimal256::from_str("24500").unwrap(),
                    last_reward_updated: current_block_height,
                },
                state
            );

            let staker_state: StakerState =
                load_staker_state(&deps.storage, &user_1_address).unwrap();
            assert_eq!(
                StakerState {
                    staked_amount: Uint256::from(deposit_1_amount),
                    reward_index: Decimal256::from_str("245").unwrap(),
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
    let casset_token = "addr0001".to_string();
    let aterra_token = "addr0002".to_string();
    let stable_denom = "uust".to_string();
    let basset_farmer_contract = "addr0003".to_string();
    let anchor_market_contract = "addr0004".to_string();

    let msg = yield_optimizer::casset_staking::InstantiateMsg {
        casset_token: casset_token.clone(),
        aterra_token: aterra_token.clone(),
        stable_denom: stable_denom.clone(),
        basset_farmer_contract: basset_farmer_contract.clone(),
        anchor_market_contract: anchor_market_contract.clone(),
    };

    let basset_farmer_loan_amount = Uint256::from(200_000u64);
    let aterra_exchange_rate = Decimal256::from_str("1.1").unwrap();
    let aterra_amount = Uint256::from(195_000u64);
    let stable_coin_balance = Uint128::from(10_000u64);

    let mut deps = mock_dependencies(&[]);
    //to: 4. get amount of UST
    deps.querier.update_base_balance(
        &basset_farmer_contract,
        vec![Coin {
            denom: stable_denom.to_string(),
            amount: stable_coin_balance,
        }],
    );

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
        deps.querier.with_token_balances(&[
            (
                &casset_token,
                &[(&MOCK_CONTRACT_ADDR.to_string(), &deposit_1_amount)],
            ),
            //to: 2. get amount of aUST
            (
                &aterra_token,
                &[(&basset_farmer_contract.clone(), &aterra_amount.into())],
            ),
        ]);
        //to: 3. get aUST to UST ratio
        deps.querier.with_wasm_query_response(&[(
            &anchor_market_contract,
            &to_binary(&AnchorMarketEpochStateResponse {
                exchange_rate: aterra_exchange_rate,
                aterra_supply: Uint256::from(1_000_000u64),
            })
            .unwrap(),
        )]);
        //to: 1. get amount of borrowed UST
        deps.querier.with_loan(&[(
            &anchor_market_contract,
            &[(
                &basset_farmer_contract.to_string(),
                &BorrowerInfoResponse {
                    borrower: basset_farmer_contract.to_string(),
                    interest_index: Decimal256::one(),
                    reward_index: Decimal256::zero(),
                    loan_amount: basset_farmer_loan_amount,
                    pending_rewards: Decimal256::zero(),
                },
            )],
        )]);

        // -= USER SEND cAsset tokens to casset_staker =-
        {
            let cw20_deposit_msg = Cw20ReceiveMsg {
                sender: user_1_address.to_string(),
                amount: deposit_1_amount,
                msg: to_binary(&Cw20HookMsg::Stake).unwrap(),
            };

            let info = mock_info(&casset_token, &vec![]);
            let current_block_height = 68882;
            let mut env = mock_env();
            env.block.height = current_block_height;
            crate::contract::execute(
                deps.as_mut(),
                env,
                info,
                ExecuteMsg::Receive(cw20_deposit_msg),
            )
            .unwrap();
        }
    }

    // -= USER SEND unstake casset tokens =-
    {
        let unstake_msg = ExecuteMsg::Anyone {
            anyone_msg: AnyoneMsg::Unstake {
                amount: Uint256::from(deposit_1_amount),
                to: None,
            },
        };

        let info = mock_info(&user_1_address.to_string(), &vec![]);
        let current_block_height = 68883;
        let mut env = mock_env();
        env.block.height = current_block_height;
        let res = crate::contract::execute(deps.as_mut(), env, info, unstake_msg).unwrap();

        assert_eq!(
            res.messages,
            vec![CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: casset_token.clone(),
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
                last_reward_amount: Decimal256::from_str("24500").unwrap(),
                last_reward_updated: current_block_height,
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
    let casset_token = "addr0001".to_string();
    let aterra_token = "addr0002".to_string();
    let stable_denom = "uust".to_string();
    let basset_farmer_contract = "addr0003".to_string();
    let anchor_market_contract = "addr0004".to_string();

    let msg = yield_optimizer::casset_staking::InstantiateMsg {
        casset_token: casset_token.clone(),
        aterra_token: aterra_token.clone(),
        stable_denom: stable_denom.clone(),
        basset_farmer_contract: basset_farmer_contract.clone(),
        anchor_market_contract: anchor_market_contract.clone(),
    };

    let basset_farmer_loan_amount = Uint256::from(200_000u64);
    let aterra_exchange_rate = Decimal256::from_str("1.1").unwrap();
    let aterra_amount = Uint256::from(195_000u64);
    let stable_coin_balance = Uint128::from(10_000u64);

    let mut deps = mock_dependencies(&[]);
    //to: 4. get amount of UST
    deps.querier.update_base_balance(
        &basset_farmer_contract,
        vec![Coin {
            denom: stable_denom.to_string(),
            amount: stable_coin_balance,
        }],
    );

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
        deps.querier.with_token_balances(&[
            (
                &casset_token,
                &[(&MOCK_CONTRACT_ADDR.to_string(), &deposit_1_amount)],
            ),
            //to: 2. get amount of aUST
            (
                &aterra_token,
                &[(&basset_farmer_contract.clone(), &aterra_amount.into())],
            ),
        ]);
        //to: 3. get aUST to UST ratio
        deps.querier.with_wasm_query_response(&[(
            &anchor_market_contract,
            &to_binary(&AnchorMarketEpochStateResponse {
                exchange_rate: aterra_exchange_rate,
                aterra_supply: Uint256::from(1_000_000u64),
            })
            .unwrap(),
        )]);
        //to: 1. get amount of borrowed UST
        deps.querier.with_loan(&[(
            &anchor_market_contract,
            &[(
                &basset_farmer_contract.to_string(),
                &BorrowerInfoResponse {
                    borrower: basset_farmer_contract.to_string(),
                    interest_index: Decimal256::one(),
                    reward_index: Decimal256::zero(),
                    loan_amount: basset_farmer_loan_amount,
                    pending_rewards: Decimal256::zero(),
                },
            )],
        )]);

        // -= USER SEND cAsset tokens to casset_staker =-
        {
            let cw20_deposit_msg = Cw20ReceiveMsg {
                sender: user_1_address.to_string(),
                amount: deposit_1_amount,
                msg: to_binary(&Cw20HookMsg::Stake).unwrap(),
            };

            let info = mock_info(&casset_token, &vec![]);
            let current_block_height = 68882;
            let mut env = mock_env();
            env.block.height = current_block_height;
            crate::contract::execute(
                deps.as_mut(),
                env,
                info,
                ExecuteMsg::Receive(cw20_deposit_msg),
            )
            .unwrap();
        }
    }

    // -= USER SEND unstake casset tokens =-
    {
        let unstake_msg = ExecuteMsg::Anyone {
            anyone_msg: AnyoneMsg::Unstake {
                amount: Uint256::from(withdraw_amount),
                to: None,
            },
        };

        let info = mock_info(&user_1_address.to_string(), &vec![]);
        let current_block_height = 68883;
        let mut env = mock_env();
        env.block.height = current_block_height;
        crate::contract::execute(deps.as_mut(), env, info, unstake_msg).unwrap();

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
    let casset_token = "addr0001".to_string();
    let aterra_token = "addr0002".to_string();
    let stable_denom = "uust".to_string();
    let basset_farmer_contract = "addr0003".to_string();
    let anchor_market_contract = "addr0004".to_string();

    let msg = yield_optimizer::casset_staking::InstantiateMsg {
        casset_token: casset_token.clone(),
        aterra_token: aterra_token.clone(),
        stable_denom: stable_denom.clone(),
        basset_farmer_contract: basset_farmer_contract.clone(),
        anchor_market_contract: anchor_market_contract.clone(),
    };

    let basset_farmer_loan_amount = Uint256::from(200_000u64);
    let aterra_exchange_rate = Decimal256::from_str("1.1").unwrap();
    let aterra_amount = Uint256::from(195_000u64);
    let stable_coin_balance = Uint128::from(10_000u64);

    let mut deps = mock_dependencies(&[]);
    //to: 4. get amount of UST
    deps.querier.update_base_balance(
        &basset_farmer_contract,
        vec![Coin {
            denom: stable_denom.to_string(),
            amount: stable_coin_balance,
        }],
    );

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
        //to: 3. get aUST to UST ratio
        deps.querier.with_wasm_query_response(&[(
            &anchor_market_contract,
            &to_binary(&AnchorMarketEpochStateResponse {
                exchange_rate: aterra_exchange_rate,
                aterra_supply: Uint256::from(1_000_000u64),
            })
            .unwrap(),
        )]);
        //to: 1. get amount of borrowed UST
        deps.querier.with_loan(&[(
            &anchor_market_contract,
            &[(
                &basset_farmer_contract.to_string(),
                &BorrowerInfoResponse {
                    borrower: basset_farmer_contract.to_string(),
                    interest_index: Decimal256::one(),
                    reward_index: Decimal256::zero(),
                    loan_amount: basset_farmer_loan_amount,
                    pending_rewards: Decimal256::zero(),
                },
            )],
        )]);

        // -= USER 1 SEND cAsset tokens to casset_staker =-
        {
            deps.querier.with_token_balances(&[
                (
                    &casset_token,
                    &[(&MOCK_CONTRACT_ADDR.to_string(), &deposit_1_amount)],
                ),
                //to: 2. get amount of aUST
                (
                    &aterra_token,
                    &[(&basset_farmer_contract.clone(), &aterra_amount.into())],
                ),
            ]);

            let cw20_deposit_msg = Cw20ReceiveMsg {
                sender: user_1_address.to_string(),
                amount: deposit_1_amount,
                msg: to_binary(&Cw20HookMsg::Stake).unwrap(),
            };

            let info = mock_info(&casset_token, &vec![]);
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

        // -= USER 2 SEND cAsset tokens to casset_staker =-
        {
            deps.querier.with_token_balances(&[
                (
                    &casset_token,
                    &[(
                        &MOCK_CONTRACT_ADDR.to_string(),
                        &(deposit_1_amount + deposit_2_amount),
                    )],
                ),
                //to: 2. get amount of aUST
                (
                    &aterra_token,
                    &[(&basset_farmer_contract.clone(), &aterra_amount.into())],
                ),
            ]);

            let cw20_deposit_msg = Cw20ReceiveMsg {
                sender: user_2_address.to_string(),
                amount: deposit_2_amount,
                msg: to_binary(&Cw20HookMsg::Stake).unwrap(),
            };

            let info = mock_info(&casset_token, &vec![]);
            let current_block_height = 68883;
            let mut env = mock_env();
            env.block.height = current_block_height;
            crate::contract::execute(
                deps.as_mut(),
                env,
                info,
                ExecuteMsg::Receive(cw20_deposit_msg),
            )
            .unwrap();

            let state: State = load_state(&deps.storage).unwrap();
            assert_eq!(
                State {
                    global_reward_index: Decimal256::from_str("245").unwrap(),
                    last_reward_amount: Decimal256::from_str("24500").unwrap(),
                    last_reward_updated: current_block_height,
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

    let new_aterra_amount = aterra_amount + Uint256::from(95_000u64);

    // -= SOME REWARDS COME (basset_farmer aUST balance changed in positive way) =-
    {
        deps.querier.with_token_balances(&[
            (
                &casset_token,
                &[(
                    &MOCK_CONTRACT_ADDR.to_string(),
                    &(deposit_1_amount + deposit_2_amount),
                )],
            ),
            //to: 2. get amount of aUST
            (
                &aterra_token,
                &[(&basset_farmer_contract.clone(), &new_aterra_amount.into())],
            ),
        ]);

        // 195 * 1.1 = 214.5 - old aUST value
        // total rewards = 214.5 + 10 - 200 = 24.5
        //
        // 290 * 1.1 = 319 - new aUST value
        // total rewards: 319 + 10 - 200 = 129
        // I didn't change loan amount or UST balance
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
        let current_block_height = 68884;
        let mut env = mock_env();
        env.block.height = current_block_height;
        let res = crate::contract::execute(deps.as_mut(), env, info, unstake_msg).unwrap();

        let state: State = load_state(&deps.storage).unwrap();
        assert_eq!(
            State {
                //104.5(new_rewards)/400(staked nAsset) = 261.25
                //plus prev reward index (245) = 506.25
                global_reward_index: Decimal256::from_str("506.25").unwrap(),
                //total rewards: 129_000
                //
                //user_1 share is 1/4, rewards after user stake: 104.5k
                //so user rewards: 104.5k/4 = 26_125
                //
                //minus claimed: 26_125
                //result: 129_000 - 26_125
                last_reward_amount: Decimal256::from_str("102875").unwrap(),
                last_reward_updated: current_block_height,
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
            vec![
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: casset_token.clone(),
                    send: vec![],
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: user_1_address.to_string(),
                        amount: withdraw_1_amount,
                    })
                    .unwrap(),
                }),
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: basset_farmer_contract.clone(),
                    send: vec![],
                    msg: to_binary(&BAssetFarmerExecuteMsg::CAssetStaker {
                        casset_staker_msg: CAssetStakerMsg::SendRewards {
                            recipient: user_1_address.to_string(),
                            //user_1 share is 1/4, rewards after user stake: 104.5k
                            //so user rewards: 104.5k/4 = 26_125
                            amount: Uint256::from(26_125u64),
                        },
                    })
                    .unwrap(),
                })
            ]
        );
        assert_eq!(res.submessages, vec![]);
    }

    // 26_125 / 1.1 = 23_750
    let aterra_sent_to_user_1_as_rewards = Uint256::from(23_750u64);
    // subtract rewards sent to user_1
    // subtract user_1 unstaked casset
    {
        deps.querier.with_token_balances(&[
            (
                &casset_token,
                &[(
                    &MOCK_CONTRACT_ADDR.to_string(),
                    &((deposit_1_amount + deposit_2_amount)
                        .checked_sub(withdraw_1_amount)
                        .unwrap()),
                )],
            ),
            //to: 2. get amount of aUST
            (
                &aterra_token,
                &[(
                    &basset_farmer_contract.clone(),
                    &(new_aterra_amount - aterra_sent_to_user_1_as_rewards).into(),
                )],
            ),
        ]);

        // 195 * 1.1 = 214.5 - old aUST value
        // total rewards = 214.5 + 10 - 200 = 24.5
        //
        // 290 * 1.1 = 319 - new aUST value
        // total rewards: 319 + 10 - 200 = 129
        // I didn't change loan amount or UST balance
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
        let current_block_height = 68885;
        let mut env = mock_env();
        env.block.height = current_block_height;
        let res = crate::contract::execute(deps.as_mut(), env, info, unstake_msg).unwrap();

        let state: State = load_state(&deps.storage).unwrap();
        assert_eq!(
            State {
                //104.5(new_rewards)/400(staked nAsset) = 261.25
                //plus prev reward index (245) = 506.25
                global_reward_index: Decimal256::from_str("506.25").unwrap(),
                //
                //user_2 share is 3/4, rewards after user stake: 104.5k
                //so user rewards: 104.5k/4 = 26_125 * 3 = 78_375
                //
                //minus claimed: 78_375
                //result: 102_875 - 78_375 = 24_500
                last_reward_amount: Decimal256::from_str("24500").unwrap(),
                last_reward_updated: current_block_height,
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
            vec![
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: casset_token.clone(),
                    send: vec![],
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: user_2_address.to_string(),
                        amount: withdraw_2_amount,
                    })
                    .unwrap(),
                }),
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: basset_farmer_contract.clone(),
                    send: vec![],
                    msg: to_binary(&BAssetFarmerExecuteMsg::CAssetStaker {
                        casset_staker_msg: CAssetStakerMsg::SendRewards {
                            recipient: user_2_address.to_string(),
                            //user_2 share is 3/4, rewards after user stake: 104.5k
                            //so user rewards: 104.5k/4 = 26_125 * 3 = 78_375
                            amount: Uint256::from(78_375u64),
                        },
                    })
                    .unwrap(),
                })
            ]
        );
        assert_eq!(res.submessages, vec![]);
    }
}

//TODO: user 1 stake, user 2 stake, some rewards come, user 1 unstake
//
//TODO: user 1 stake, some rewards come, user 2 stake
//TODO: basset_farmer balance changed in negative way
//TODO: claim rewards test (in new file)
//TODO: first user comes, but there is already some rewards! Should we gave them to him, or what?
