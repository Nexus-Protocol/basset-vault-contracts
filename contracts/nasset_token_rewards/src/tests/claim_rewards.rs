use crate::state::{load_holder, load_state, Holder, State};
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
use yield_optimizer::nasset_token_rewards::{AnyoneMsg, ExecuteMsg};

fn initialize_token(token_addr: String) {}

#[test]
fn claim_non_zero_rewards() {
    let psi_token_addr = "addr0001".to_string();
    let nasset_token_addr = "addr0002".to_string();
    let governance_contract_addr = "addr0003".to_string();

    let msg = yield_optimizer::nasset_token_rewards::InstantiateMsg {
        psi_token_addr,
        nasset_token_addr,
        governance_contract_addr,
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

        // -= USER SEND nAsset tokens to nasset_rewards =-
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

        // -= USER SEND CLAIM message =-
        {
            let claim_msg = ExecuteMsg::Anyone {
                anyone_msg: AnyoneMsg::ClaimRewards { to: None },
            };
            let info = mock_info(&user_1_address.to_string(), &vec![]);
            let res = crate::contract::execute(deps.as_mut(), mock_env(), info, claim_msg).unwrap();

            assert_eq!(
                res.messages,
                vec![CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: psi_token.clone(),
                    send: vec![],
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: user_1_address.to_string(),
                        amount: (rewards_after_stake + rewards_before_stake).into(),
                    })
                    .unwrap(),
                })]
            );
            assert!(res.submessages.is_empty());

            let state: State = load_state(&deps.storage).unwrap();
            assert_eq!(
                State {
                    global_reward_index: Decimal256::from_str("60").unwrap(),
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
                    reward_index: Decimal256::from_str("60").unwrap(),
                    pending_rewards: Decimal256::zero(),
                },
                staker_state
            );
        }
    }
}

#[test]
fn claim_zero_rewards() {
    let nasset_token = "addr0001".to_string();
    let psi_token = "addr0002".to_string();
    let governance_contract = "addr0003".to_string();

    let msg = yield_optimizer::nasset_rewards::InstantiateMsg {
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
    let rewards: Uint128 = Uint128::zero();
    {
        deps.querier.with_token_balances(&[
            (
                &nasset_token,
                &[(&MOCK_CONTRACT_ADDR.to_string(), &deposit_1_amount)],
            ),
            (&psi_token, &[(&MOCK_CONTRACT_ADDR.to_string(), &rewards)]),
        ]);

        // -= USER SEND nAsset tokens to nasset_rewards =-
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
            (&psi_token, &[(&MOCK_CONTRACT_ADDR.to_string(), &rewards)]),
        ]);

        // -= USER SEND CLAIM message =-
        {
            let claim_msg = ExecuteMsg::Anyone {
                anyone_msg: AnyoneMsg::ClaimRewards { to: None },
            };
            let info = mock_info(&user_1_address.to_string(), &vec![]);
            let res = crate::contract::execute(deps.as_mut(), mock_env(), info, claim_msg).unwrap();

            assert!(res.messages.is_empty());
            assert!(res.submessages.is_empty());

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
