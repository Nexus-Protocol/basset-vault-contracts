use super::sdk::Sdk;
use crate::{
    error::ContractError,
    tests::sdk::{GOVERNANCE_CONTRACT_ADDR, NASSET_TOKEN_REWARDS_CONTRACT_ADDR, PSI_TOKEN_ADDR},
};
use cosmwasm_std::{to_binary, StdError, Uint128};
use cosmwasm_std::{CosmosMsg, WasmMsg};
use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;

#[test]
fn distribute_rewards() {
    let mut sdk = Sdk::init();

    //contract have some "rewards"
    sdk.set_psi_balance(Uint128(1000));

    //===============================================================================
    //distribute rewards

    let response = sdk.distribute_rewards().unwrap();
    assert_eq!(
        response.messages,
        vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: PSI_TOKEN_ADDR.to_string(),
                send: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: NASSET_TOKEN_REWARDS_CONTRACT_ADDR.to_string(),
                    amount: Uint128::from(700u64),
                })
                .unwrap(),
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: PSI_TOKEN_ADDR.to_string(),
                send: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: GOVERNANCE_CONTRACT_ADDR.to_string(),
                    amount: Uint128::from(300u64),
                })
                .unwrap(),
            }),
        ]
    );
    //===============================================================================
}

#[test]
fn distribute_rewards_with_zero_balance() {
    let mut sdk = Sdk::init();

    //===============================================================================
    //distribute rewards

    let response = sdk.distribute_rewards();
    assert!(response.is_err());
    let error = response.err().unwrap();
    if let ContractError::Std(StdError::GenericErr { msg }) = error {
        assert_eq!("psi balance is zero", msg);
    } else {
        panic!("wrong error");
    }
    //===============================================================================
}
