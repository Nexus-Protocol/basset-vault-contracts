use crate::error::ContractError;
use crate::state::load_config;
use crate::tests::sdk::GOVERNANCE_CONTRACT_ADDR;

use super::sdk::Sdk;
use cosmwasm_std::testing::{mock_env, mock_info};
use yield_optimizer::psi_distributor::{ExecuteMsg, GovernanceMsg};

#[test]
fn fail_to_change_config_if_sender_is_not_governance() {
    let mut sdk = Sdk::init();

    let change_config_msg = ExecuteMsg::GovernanceMsg {
        governance_msg: yield_optimizer::psi_distributor::GovernanceMsg::UpdateConfig {
            psi_token_contract_addr: Some("addr9999".to_string()),
            governance_contract_addr: Some("addr9997".to_string()),
        },
    };

    let env = mock_env();
    let info = mock_info("addr0010", &[]);
    let res = crate::contract::execute(sdk.deps.as_mut(), env, info, change_config_msg);
    assert!(res.is_err());
    assert_eq!(ContractError::Unauthorized, res.err().unwrap());
}

#[test]
fn success_to_change_config_if_sender_governance() {
    let mut sdk = Sdk::init();

    let new_psi_token_contract_addr = "addr9999".to_string();
    let new_governance_contract_addr = "addr9997".to_string();

    let change_config_msg = ExecuteMsg::GovernanceMsg {
        governance_msg: GovernanceMsg::UpdateConfig {
            psi_token_contract_addr: Some(new_psi_token_contract_addr.clone()),
            governance_contract_addr: Some(new_governance_contract_addr.clone()),
        },
    };

    let env = mock_env();
    let info = mock_info(GOVERNANCE_CONTRACT_ADDR, &[]);
    crate::contract::execute(sdk.deps.as_mut(), env, info, change_config_msg).unwrap();

    let config = load_config(&sdk.deps.storage).unwrap();
    assert_eq!(new_psi_token_contract_addr, config.psi_token);
    assert_eq!(new_governance_contract_addr, config.governance_contract);
}
