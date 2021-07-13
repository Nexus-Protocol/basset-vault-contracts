use crate::error::ContractError;
use crate::state::load_config;

use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::testing::{mock_env, mock_info};
use yield_optimizer::nasset_token_config_holder::{ExecuteMsg, GovernanceMsg};

#[test]
fn fail_to_change_config_if_sender_is_not_governance() {
    let mut deps = mock_dependencies(&[]);

    let msg = yield_optimizer::nasset_token_config_holder::InstantiateMsg {
        governance_contract_addr: "addr0001".to_string(),
    };

    let env = mock_env();
    let info = mock_info("addr0010", &[]);
    crate::contract::instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

    // ====================================
    // ====================================
    // ====================================

    let new_nasset_token_rewards_contract_addr = Some("addr9999".to_string());
    let new_governance_contract_addr = Some("addr9998".to_string());

    let change_config_msg = ExecuteMsg::Governance {
        governance_msg: GovernanceMsg::UpdateConfig {
            nasset_token_rewards_contract_addr: new_nasset_token_rewards_contract_addr,
            governance_contract_addr: new_governance_contract_addr,
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
    let governance_contract_addr = "addr0000".to_string();

    let msg = yield_optimizer::nasset_token_config_holder::InstantiateMsg {
        governance_contract_addr: governance_contract_addr.clone(),
    };

    let env = mock_env();
    let info = mock_info("addr0010", &[]);
    crate::contract::instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

    // ====================================
    // ====================================
    // ====================================

    let new_nasset_token_rewards_contract_addr = "addr9999".to_string();
    let new_governance_contract_addr = "addr9998".to_string();

    let change_config_msg = ExecuteMsg::Governance {
        governance_msg: GovernanceMsg::UpdateConfig {
            nasset_token_rewards_contract_addr: Some(
                new_nasset_token_rewards_contract_addr.clone(),
            ),
            governance_contract_addr: Some(new_governance_contract_addr.clone()),
        },
    };

    let env = mock_env();
    let info = mock_info(&governance_contract_addr, &[]);
    crate::contract::execute(deps.as_mut(), env, info, change_config_msg).unwrap();

    let config = load_config(&deps.storage).unwrap();
    assert_eq!(new_governance_contract_addr, config.governance_contract);
    assert_eq!(
        new_nasset_token_rewards_contract_addr,
        config.nasset_token_rewards_contract
    );
}
