use crate::state::load_config;
use crate::tests::sdk::GOVERNANCE_CONTRACT;

use super::sdk::Sdk;
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::StdError;
use yield_optimizer::basset_vault::{ExecuteMsg, GovernanceMsg};

#[test]
fn fail_to_change_config_if_sender_is_not_governance() {
    let mut sdk = Sdk::init();

    let change_config_msg = ExecuteMsg::Governance {
        governance_msg: GovernanceMsg::UpdateConfig {
            psi_distributor_addr: Some("addr9998".to_string()),
        },
    };

    let env = mock_env();
    let info = mock_info("addr0010", &[]);
    let res = crate::contract::execute(sdk.deps.as_mut(), env, info, change_config_msg);
    assert!(res.is_err());
    if let StdError::GenericErr { msg } = res.err().unwrap() {
        assert_eq!("unauthhorized", msg);
    } else {
        panic!("wrong error");
    }
}

#[test]
fn success_to_change_config_if_sender_governance() {
    let mut sdk = Sdk::init();

    let new_psi_distributor_addr = "addr9998".to_string();

    let change_config_msg = ExecuteMsg::Governance {
        governance_msg: GovernanceMsg::UpdateConfig {
            psi_distributor_addr: Some(new_psi_distributor_addr.clone()),
        },
    };

    let env = mock_env();
    let info = mock_info(GOVERNANCE_CONTRACT, &[]);
    crate::contract::execute(sdk.deps.as_mut(), env, info, change_config_msg).unwrap();

    let config = load_config(&sdk.deps.storage).unwrap();
    assert_eq!(new_psi_distributor_addr, config.psi_distributor);
}
