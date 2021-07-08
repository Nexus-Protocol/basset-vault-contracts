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
        NASSET_TOKEN_ADDR, NASSET_TOKEN_CODE_ID, NASSET_TOKEN_CONFIG_HOLDER_CODE_ID,
        NASSET_TOKEN_CONFIG_HOLDER_CONTRACT, NASSET_TOKEN_REWARDS_CODE_ID,
        NASSET_TOKEN_REWARDS_CONTRACT, OVER_LOAN_BALANCE_VALUE, PSI_DISTRIBUTOR_CODE_ID,
        PSI_DISTRIBUTOR_CONTRACT, PSI_STABLE_SWAP_CONTRACT, PSI_TOKEN, STABLE_DENOM,
    },
};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR},
    CosmosMsg,
};
use cosmwasm_std::{
    to_binary, Addr, Coin, ContractResult, Decimal, Reply, ReplyOn, Response, SubMsg,
    SubcallResponse, Uint128, WasmMsg,
};
use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;
use std::str::FromStr;
use yield_optimizer::{
    basset_farmer::{AnyoneMsg, ExecuteMsg},
    basset_farmer_config::BorrowerActionResponse,
    querier::{
        AnchorMarketCw20Msg, AnchorMarketEpochStateResponse, AnchorMarketMsg, AnchorOverseerMsg,
        BorrowerInfoResponse,
    },
};

#[test]
fn deposit_basset() {
    let mut sdk = Sdk::init();

    //first farmer come
    let user_1_address = "addr9999".to_string();
    let deposit_1_amount: Uint256 = 2_000_000_000u128.into();
    {
        // -= USER SEND bAsset tokens to basset_farmer =-
        sdk.set_nasset_supply(Uint256::zero());
        sdk.set_basset_balance(deposit_1_amount);

        let response = sdk
            .user_deposit(&user_1_address, deposit_1_amount.into())
            .unwrap();

        assert_eq!(
            response.messages,
            vec![
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: ANCHOR_OVERSEER_CONTRACT.to_string(),
                    msg: to_binary(&AnchorOverseerMsg::LockCollateral {
                        collaterals: vec![(BASSET_TOKEN_ADDR.to_string(), deposit_1_amount)],
                    })
                    .unwrap(),
                    send: vec![],
                }),
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: NASSET_TOKEN_ADDR.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Mint {
                        recipient: user_1_address.clone(),
                        amount: deposit_1_amount.into(), //first depositer have same amount
                    })
                    .unwrap(),
                    send: vec![],
                }),
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: MOCK_CONTRACT_ADDR.to_string(),
                    msg: to_binary(&ExecuteMsg::Anyone {
                        anyone_msg: AnyoneMsg::Rebalance,
                    })
                    .unwrap(),
                    send: vec![],
                }),
            ]
        );
    }

    //second farmer come
    let user_2_address = "addr6666".to_string();
    let deposit_2_amount: Uint256 = 6_000_000_000u128.into();
    {
        sdk.set_nasset_supply(deposit_1_amount);
        sdk.set_collateral_balance(deposit_1_amount, Uint256::zero());
        sdk.set_basset_balance(deposit_2_amount);
        // -= USER SEND bAsset tokens to basset_farmer =-
        let response = sdk
            .user_deposit(&user_2_address, deposit_2_amount.into())
            .unwrap();
        assert_eq!(
            response.messages,
            vec![
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: ANCHOR_OVERSEER_CONTRACT.to_string(),
                    msg: to_binary(&AnchorOverseerMsg::LockCollateral {
                        collaterals: vec![(BASSET_TOKEN_ADDR.to_string(), deposit_2_amount)],
                    })
                    .unwrap(),
                    send: vec![],
                }),
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: NASSET_TOKEN_ADDR.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Mint {
                        recipient: user_2_address.clone(),
                        amount: Uint128(6_000_000_000), //2B * (6B/8B) / (1 - (6B/8B)) = 6B
                    })
                    .unwrap(),
                    send: vec![],
                }),
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: MOCK_CONTRACT_ADDR.to_string(),
                    msg: to_binary(&ExecuteMsg::Anyone {
                        anyone_msg: AnyoneMsg::Rebalance,
                    })
                    .unwrap(),
                    send: vec![],
                }),
            ]
        );
    }
}
