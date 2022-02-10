use crate::error::ContractError;
use crate::state::{load_config, load_gov_update};

use basset_vault::basset_vault_strategy::{AnyoneMsg, ExecuteMsg, GovernanceMsg};
use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{Addr, BlockInfo, StdError};
use std::str::FromStr;

#[test]
fn fail_to_change_governance_if_sender_is_not_governance() {
    let mut deps = mock_dependencies(&[]);
    let governance_contract_addr = "addr0000".to_string();

    let msg = basset_vault::basset_vault_strategy::InstantiateMsg {
        governance_contract_addr: governance_contract_addr.clone(),
        oracle_contract_addr: "addr0001".to_string(),
        anchor_market_addr: "addr0002".to_string(),
        anchor_interest_model_addr: "addr0003".to_string(),
        anchor_overseer_addr: "addr0004".to_string(),
        anc_ust_swap_addr: "addr0006".to_string(),
        basset_token_addr: "addr0007".to_string(),
        stable_denom: "uust".to_string(),
        borrow_ltv_max: Decimal256::from_str("0.85").unwrap(),
        borrow_ltv_min: Decimal256::from_str("0.75").unwrap(),
        borrow_ltv_aim: Decimal256::from_str("0.8").unwrap(),
        basset_max_ltv: Decimal256::from_str("0.5").unwrap(),
        buffer_part: Decimal256::from_str("0.018").unwrap(),
        price_timeframe: 60,
    };

    {
        let env = mock_env();
        let info = mock_info("addr0010", &[]);
        crate::contract::instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();
    }

    // ====================================
    // ====================================
    // ====================================

    let change_gov_msg = ExecuteMsg::Governance {
        governance_msg: GovernanceMsg::UpdateGovernanceContract {
            gov_addr: "addr9998".to_string(),
            seconds_to_wait_for_accept_gov_tx: 60,
        },
    };

    let env = mock_env();
    let info = mock_info("addr0010", &[]);
    let res = crate::contract::execute(deps.as_mut(), env, info, change_gov_msg);
    assert!(res.is_err());
    let error_value = res.err().unwrap();
    assert_eq!(ContractError::Unauthorized, error_value);
}

#[test]
fn success_to_change_governance_if_sender_governance() {
    let mut deps = mock_dependencies(&[]);
    let governance_contract_addr = "addr0000".to_string();

    let msg = basset_vault::basset_vault_strategy::InstantiateMsg {
        governance_contract_addr: governance_contract_addr.clone(),
        oracle_contract_addr: "addr0001".to_string(),
        anchor_market_addr: "addr0002".to_string(),
        anchor_interest_model_addr: "addr0003".to_string(),
        anchor_overseer_addr: "addr0004".to_string(),
        anc_ust_swap_addr: "addr0006".to_string(),
        basset_token_addr: "addr0007".to_string(),
        stable_denom: "uust".to_string(),
        borrow_ltv_max: Decimal256::from_str("0.85").unwrap(),
        borrow_ltv_min: Decimal256::from_str("0.75").unwrap(),
        borrow_ltv_aim: Decimal256::from_str("0.8").unwrap(),
        basset_max_ltv: Decimal256::from_str("0.5").unwrap(),
        buffer_part: Decimal256::from_str("0.018").unwrap(),
        price_timeframe: 60,
    };

    {
        let env = mock_env();
        let info = mock_info("addr0010", &[]);
        crate::contract::instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();
    }

    // ====================================
    // ====================================
    // ====================================

    let new_gov_addr = "addr9994".to_string();
    let seconds_to_wait_for_accept_gov_tx = 60;

    // Send message to change governance address
    {
        let change_gov_msg = ExecuteMsg::Governance {
            governance_msg: GovernanceMsg::UpdateGovernanceContract {
                gov_addr: new_gov_addr.clone(),
                seconds_to_wait_for_accept_gov_tx,
            },
        };

        let env = mock_env();
        let info = mock_info(&governance_contract_addr.clone(), &[]);
        crate::contract::execute(deps.as_mut(), env.clone(), info, change_gov_msg).unwrap();

        let gov_update_state = load_gov_update(&deps.storage).unwrap();
        assert_eq!(
            Addr::unchecked(new_gov_addr.clone()),
            gov_update_state.new_governance_contract_addr
        );
        assert_eq!(
            seconds_to_wait_for_accept_gov_tx + get_time(&env.block),
            gov_update_state.wait_approve_until
        );
    }

    // Send message to accept governance
    {
        let accept_gov_msg = ExecuteMsg::Anyone {
            anyone_msg: AnyoneMsg::AcceptGovernance {},
        };

        let mut env = mock_env();
        env.block.time = env.block.time.plus_seconds(20);
        let info = mock_info(&new_gov_addr, &[]);
        crate::contract::execute(deps.as_mut(), env.clone(), info, accept_gov_msg).unwrap();

        let gov_update_state = load_gov_update(&deps.storage);
        assert!(gov_update_state.is_err());

        let config = load_config(&deps.storage).unwrap();
        assert_eq!(config.governance_contract, new_gov_addr);
    }
}

#[test]
fn fail_to_accept_governance_if_sender_is_wrong() {
    let mut deps = mock_dependencies(&[]);
    let governance_contract_addr = "addr0000".to_string();

    let msg = basset_vault::basset_vault_strategy::InstantiateMsg {
        governance_contract_addr: governance_contract_addr.clone(),
        oracle_contract_addr: "addr0001".to_string(),
        anchor_market_addr: "addr0002".to_string(),
        anchor_interest_model_addr: "addr0003".to_string(),
        anchor_overseer_addr: "addr0004".to_string(),
        anc_ust_swap_addr: "addr0006".to_string(),
        basset_token_addr: "addr0007".to_string(),
        stable_denom: "uust".to_string(),
        borrow_ltv_max: Decimal256::from_str("0.85").unwrap(),
        borrow_ltv_min: Decimal256::from_str("0.75").unwrap(),
        borrow_ltv_aim: Decimal256::from_str("0.8").unwrap(),
        basset_max_ltv: Decimal256::from_str("0.5").unwrap(),
        buffer_part: Decimal256::from_str("0.018").unwrap(),
        price_timeframe: 60,
    };

    {
        let env = mock_env();
        let info = mock_info("addr0010", &[]);
        crate::contract::instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();
    }

    // ====================================
    // ====================================
    // ====================================

    let new_gov_addr = "addr9994".to_string();
    let random_sender = "anyone".to_string();
    let seconds_to_wait_for_accept_gov_tx = 60;

    // Send message to change governance address
    {
        let change_gov_msg = ExecuteMsg::Governance {
            governance_msg: GovernanceMsg::UpdateGovernanceContract {
                gov_addr: new_gov_addr.clone(),
                seconds_to_wait_for_accept_gov_tx,
            },
        };

        let env = mock_env();
        let info = mock_info(&governance_contract_addr.clone(), &[]);
        crate::contract::execute(deps.as_mut(), env.clone(), info, change_gov_msg).unwrap();

        let gov_update_state = load_gov_update(&deps.storage).unwrap();
        assert_eq!(
            Addr::unchecked(new_gov_addr.clone()),
            gov_update_state.new_governance_contract_addr
        );
        assert_eq!(
            seconds_to_wait_for_accept_gov_tx + get_time(&env.block),
            gov_update_state.wait_approve_until
        );
    }

    // Send message to accept governance from wrong address
    {
        let accept_gov_msg = ExecuteMsg::Anyone {
            anyone_msg: AnyoneMsg::AcceptGovernance {},
        };

        let mut env = mock_env();
        env.block.time = env.block.time.plus_seconds(20);
        let info = mock_info(&random_sender, &[]);
        let gov_update_state_res =
            crate::contract::execute(deps.as_mut(), env.clone(), info, accept_gov_msg);

        assert!(gov_update_state_res.is_err());
        let error_value = gov_update_state_res.err().unwrap();
        assert_eq!(ContractError::Unauthorized, error_value);
    }
}

#[test]
fn too_late_to_change_governance() {
    let mut deps = mock_dependencies(&[]);
    let governance_contract_addr = "addr0000".to_string();

    let msg = basset_vault::basset_vault_strategy::InstantiateMsg {
        governance_contract_addr: governance_contract_addr.clone(),
        oracle_contract_addr: "addr0001".to_string(),
        anchor_market_addr: "addr0002".to_string(),
        anchor_interest_model_addr: "addr0003".to_string(),
        anchor_overseer_addr: "addr0004".to_string(),
        anc_ust_swap_addr: "addr0006".to_string(),
        basset_token_addr: "addr0007".to_string(),
        stable_denom: "uust".to_string(),
        borrow_ltv_max: Decimal256::from_str("0.85").unwrap(),
        borrow_ltv_min: Decimal256::from_str("0.75").unwrap(),
        borrow_ltv_aim: Decimal256::from_str("0.8").unwrap(),
        basset_max_ltv: Decimal256::from_str("0.5").unwrap(),
        buffer_part: Decimal256::from_str("0.018").unwrap(),
        price_timeframe: 60,
    };

    {
        let env = mock_env();
        let info = mock_info("addr0010", &[]);
        crate::contract::instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();
    }

    // ====================================
    // ====================================
    // ====================================

    let new_gov_addr = "addr9994".to_string();
    let seconds_to_wait_for_accept_gov_tx = 60;

    // Send message to change governance address
    {
        let change_gov_msg = ExecuteMsg::Governance {
            governance_msg: GovernanceMsg::UpdateGovernanceContract {
                gov_addr: new_gov_addr.clone(),
                seconds_to_wait_for_accept_gov_tx,
            },
        };

        let env = mock_env();
        let info = mock_info(&governance_contract_addr.clone(), &[]);
        crate::contract::execute(deps.as_mut(), env.clone(), info, change_gov_msg).unwrap();

        let gov_update_state = load_gov_update(&deps.storage).unwrap();
        assert_eq!(
            Addr::unchecked(new_gov_addr.clone()),
            gov_update_state.new_governance_contract_addr
        );
        assert_eq!(
            seconds_to_wait_for_accept_gov_tx + get_time(&env.block),
            gov_update_state.wait_approve_until
        );
    }

    // Send message to accept governance
    {
        let accept_gov_msg = ExecuteMsg::Anyone {
            anyone_msg: AnyoneMsg::AcceptGovernance {},
        };

        let mut env = mock_env();
        env.block.time = env
            .block
            .time
            .plus_seconds(seconds_to_wait_for_accept_gov_tx + 1);
        let info = mock_info(&new_gov_addr, &[]);
        let accept_gov_res =
            crate::contract::execute(deps.as_mut(), env.clone(), info, accept_gov_msg);

        assert!(accept_gov_res.is_err());
        if let ContractError::Std(std_error) = accept_gov_res.err().unwrap() {
            match std_error {
                StdError::GenericErr { msg, .. } => {
                    assert_eq!("too late to accept governance owning", msg);
                }

                _ => panic!("wrong error"),
            }
        } else {
            panic!("wrong error");
        }
    }
}

#[test]
fn rewrite_new_gov_address_by_sending_second_update_gov_message() {
    let mut deps = mock_dependencies(&[]);
    let governance_contract_addr = "addr0000".to_string();

    let msg = basset_vault::basset_vault_strategy::InstantiateMsg {
        governance_contract_addr: governance_contract_addr.clone(),
        oracle_contract_addr: "addr0001".to_string(),
        anchor_market_addr: "addr0002".to_string(),
        anchor_interest_model_addr: "addr0003".to_string(),
        anchor_overseer_addr: "addr0004".to_string(),
        anc_ust_swap_addr: "addr0006".to_string(),
        basset_token_addr: "addr0007".to_string(),
        stable_denom: "uust".to_string(),
        borrow_ltv_max: Decimal256::from_str("0.85").unwrap(),
        borrow_ltv_min: Decimal256::from_str("0.75").unwrap(),
        borrow_ltv_aim: Decimal256::from_str("0.8").unwrap(),
        basset_max_ltv: Decimal256::from_str("0.5").unwrap(),
        buffer_part: Decimal256::from_str("0.018").unwrap(),
        price_timeframe: 60,
    };

    {
        let env = mock_env();
        let info = mock_info("addr0010", &[]);
        crate::contract::instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();
    }

    // ====================================
    // ====================================
    // ====================================

    let new_gov_addr = "addr9994".to_string();
    let wrong_gov_address = "wrong_address".to_string();
    let seconds_to_wait_for_accept_gov_tx = 60;

    // Send message to change governance address
    {
        let change_gov_msg = ExecuteMsg::Governance {
            governance_msg: GovernanceMsg::UpdateGovernanceContract {
                gov_addr: wrong_gov_address.clone(),
                seconds_to_wait_for_accept_gov_tx,
            },
        };

        let env = mock_env();
        let info = mock_info(&governance_contract_addr.clone(), &[]);
        crate::contract::execute(deps.as_mut(), env.clone(), info, change_gov_msg).unwrap();

        let gov_update_state = load_gov_update(&deps.storage).unwrap();
        assert_eq!(
            Addr::unchecked(wrong_gov_address),
            gov_update_state.new_governance_contract_addr
        );
        assert_eq!(
            seconds_to_wait_for_accept_gov_tx + get_time(&env.block),
            gov_update_state.wait_approve_until
        );
    }

    // Send second message to change governance address
    {
        let change_gov_msg = ExecuteMsg::Governance {
            governance_msg: GovernanceMsg::UpdateGovernanceContract {
                gov_addr: new_gov_addr.clone(),
                seconds_to_wait_for_accept_gov_tx,
            },
        };

        let mut env = mock_env();
        env.block.time = env.block.time.plus_seconds(22);
        let info = mock_info(&governance_contract_addr.clone(), &[]);
        crate::contract::execute(deps.as_mut(), env.clone(), info, change_gov_msg).unwrap();

        let gov_update_state = load_gov_update(&deps.storage).unwrap();
        assert_eq!(
            Addr::unchecked(new_gov_addr.clone()),
            gov_update_state.new_governance_contract_addr
        );
        assert_eq!(
            seconds_to_wait_for_accept_gov_tx + get_time(&env.block),
            gov_update_state.wait_approve_until
        );
    }
}

fn get_time(block: &BlockInfo) -> u64 {
    block.time.seconds()
}
