use crate::state::{load_config, load_gov_update};
use crate::tests::sdk::GOVERNANCE_CONTRACT;

use super::sdk::Sdk;
use basset_vault::basset_vault::{AnyoneMsg, ExecuteMsg, GovernanceMsg};
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{Addr, BlockInfo, StdError};

#[test]
fn fail_to_change_governance_if_sender_is_not_governance() {
    let mut sdk = Sdk::init();

    let change_gov_msg = ExecuteMsg::Governance {
        governance_msg: GovernanceMsg::UpdateGovernanceContract {
            gov_addr: "addr9998".to_string(),
            seconds_to_wait_for_accept_gov_tx: 60,
        },
    };

    let env = mock_env();
    let info = mock_info("addr0010", &[]);
    let res = crate::contract::execute(sdk.deps.as_mut(), env, info, change_gov_msg);
    assert!(res.is_err());
    if let StdError::GenericErr { msg, .. } = res.err().unwrap() {
        assert_eq!("unauthorized", msg);
    } else {
        panic!("wrong error");
    }
}

#[test]
fn success_to_change_governance_if_sender_governance() {
    let mut sdk = Sdk::init();

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
        let info = mock_info(GOVERNANCE_CONTRACT, &[]);
        crate::contract::execute(sdk.deps.as_mut(), env.clone(), info, change_gov_msg).unwrap();

        let gov_update_state = load_gov_update(&sdk.deps.storage).unwrap();
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
        crate::contract::execute(sdk.deps.as_mut(), env.clone(), info, accept_gov_msg).unwrap();

        let gov_update_state = load_gov_update(&sdk.deps.storage);
        assert!(gov_update_state.is_err());

        let config = load_config(&sdk.deps.storage).unwrap();
        assert_eq!(config.governance_contract, new_gov_addr);
    }
}

#[test]
fn fail_to_accept_governance_if_sender_is_wrong() {
    let mut sdk = Sdk::init();

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
        let info = mock_info(GOVERNANCE_CONTRACT, &[]);
        crate::contract::execute(sdk.deps.as_mut(), env.clone(), info, change_gov_msg).unwrap();

        let gov_update_state = load_gov_update(&sdk.deps.storage).unwrap();
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
            crate::contract::execute(sdk.deps.as_mut(), env.clone(), info, accept_gov_msg);

        assert!(gov_update_state_res.is_err());
        if let StdError::GenericErr { msg, .. } = gov_update_state_res.err().unwrap() {
            assert_eq!("unauthorized", msg);
        } else {
            panic!("wrong error");
        }
    }
}

#[test]
fn too_late_to_change_governance() {
    let mut sdk = Sdk::init();

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
        let info = mock_info(GOVERNANCE_CONTRACT, &[]);
        crate::contract::execute(sdk.deps.as_mut(), env.clone(), info, change_gov_msg).unwrap();

        let gov_update_state = load_gov_update(&sdk.deps.storage).unwrap();
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
            crate::contract::execute(sdk.deps.as_mut(), env.clone(), info, accept_gov_msg);

        assert!(accept_gov_res.is_err());
        if let StdError::GenericErr { msg, .. } = accept_gov_res.err().unwrap() {
            assert_eq!("too late to accept governance owning", msg);
        } else {
            panic!("wrong error");
        }
    }
}

#[test]
fn rewrite_new_gov_address_by_sending_second_update_gov_message() {
    let mut sdk = Sdk::init();

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
        let info = mock_info(GOVERNANCE_CONTRACT, &[]);
        crate::contract::execute(sdk.deps.as_mut(), env.clone(), info, change_gov_msg).unwrap();

        let gov_update_state = load_gov_update(&sdk.deps.storage).unwrap();
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
        let info = mock_info(GOVERNANCE_CONTRACT, &[]);
        crate::contract::execute(sdk.deps.as_mut(), env.clone(), info, change_gov_msg).unwrap();

        let gov_update_state = load_gov_update(&sdk.deps.storage).unwrap();
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
