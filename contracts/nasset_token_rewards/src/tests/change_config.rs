use crate::error::ContractError;
use crate::state::load_config;

use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::testing::{mock_env, mock_info};
use yield_optimizer::nasset_token_rewards::{ExecuteMsg, GovernanceMsg};

#[test]
fn fail_to_change_config_if_sender_is_not_governance() {
    let mut deps = mock_dependencies(&[]);

    let msg = yield_optimizer::nasset_token_rewards::InstantiateMsg {
        psi_token_addr: "addr0000".to_string(),
        nasset_token_addr: "addr0001".to_string(),
        governance_contract_addr: "addr0002".to_string(),
    };

    let env = mock_env();
    let info = mock_info("addr0010", &[]);
    crate::contract::instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();

    let change_config_msg = ExecuteMsg::Governance {
        governance_msg: yield_optimizer::nasset_token_rewards::GovernanceMsg::UpdateConfig {
            psi_token_contract_addr: Some("addr9999".to_string()),
            nasset_token_contract_addr: Some("addr9998".to_string()),
            governance_contract_addr: Some("addr9997".to_string()),
        },
    };

    let env = mock_env();
    let info = mock_info("addr0010", &[]);
    let res = crate::contract::execute(deps.as_mut(), env, info, change_config_msg);
    assert!(res.is_err());
    assert_eq!(ContractError::Unauthorized, res.err().unwrap());
}

#[test]
fn success_to_change_config_if_sender_governance() {
    let mut deps = mock_dependencies(&[]);
    let governance_contract_addr = "addr0002".to_string();

    let msg = yield_optimizer::nasset_token_rewards::InstantiateMsg {
        psi_token_addr: "addr0000".to_string(),
        nasset_token_addr: "addr0001".to_string(),
        governance_contract_addr: governance_contract_addr.clone(),
    };

    let env = mock_env();
    let info = mock_info("addr0010", &[]);
    crate::contract::instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();

    let new_psi_token_contract_addr = "addr9999".to_string();
    let new_nasset_token_contract_addr = "addr9998".to_string();
    let new_governance_contract_addr = "addr9997".to_string();

    let change_config_msg = ExecuteMsg::Governance {
        governance_msg: GovernanceMsg::UpdateConfig {
            psi_token_contract_addr: Some(new_psi_token_contract_addr.clone()),
            nasset_token_contract_addr: Some(new_nasset_token_contract_addr.clone()),
            governance_contract_addr: Some(new_governance_contract_addr.clone()),
        },
    };

    let env = mock_env();
    let info = mock_info(&governance_contract_addr, &[]);
    crate::contract::execute(deps.as_mut(), env, info, change_config_msg).unwrap();

    let config = load_config(&deps.storage).unwrap();
    assert_eq!(new_psi_token_contract_addr, config.psi_token);
    assert_eq!(new_nasset_token_contract_addr, config.nasset_token);
    assert_eq!(new_governance_contract_addr, config.governance_contract);
}
