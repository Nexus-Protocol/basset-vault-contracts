use crate::state::{load_config, load_staker_state, load_state, Config, StakerState, State};
use crate::tests::mock_dependencies;
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR},
    Addr, Coin, Decimal,
};
use cosmwasm_std::{to_binary, Uint128};
use cw20::Cw20ReceiveMsg;
use std::str::FromStr;
use yield_optimizer::{
    casset_staking::{Cw20HookMsg, ExecuteMsg},
    querier::{AnchorMarketEpochStateResponse, BorrowerInfoResponse},
};

// borrowed_amount: 200000, aterra_amount: 195000,
// aterra_exchange_rate: 1.1, stable_coin_amount: 10000, casset_staked_amount: 100
//
// borrowed_amount: 200000, aterra_amount: 195000,
// aterra_exchange_rate: 1.1, stable_coin_amount: 10000, casset_staked_amount: 100
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
