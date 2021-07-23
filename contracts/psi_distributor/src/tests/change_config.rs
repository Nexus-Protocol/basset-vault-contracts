use crate::error::ContractError;
use crate::state::load_config;
use crate::tests::sdk::GOVERNANCE_CONTRACT_ADDR;

use super::sdk::Sdk;
use basset_vault::psi_distributor::{ExecuteMsg, GovernanceMsg};
use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::testing::{mock_env, mock_info};
use std::str::FromStr;

#[test]
fn fail_to_change_config_if_sender_is_not_governance() {
    let mut sdk = Sdk::init();

    let change_config_msg = ExecuteMsg::GovernanceMsg {
        governance_msg: basset_vault::psi_distributor::GovernanceMsg::UpdateConfig {
            governance_contract_addr: Some("addr9997".to_string()),
            nasset_token_rewards_contract_addr: None,
            community_pool_contract_addr: None,
            basset_vault_strategy_contract_addr: None,
            manual_ltv: None,
            fee_rate: None,
            tax_rate: None,
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

    let new_governance_contract_addr = "addr9997".to_string();
    let new_nasset_token_rewards_contract_addr = "addr9996".to_string();
    let new_basset_vault_strategy_contract_addr = "addr9995".to_string();
    let new_community_pool_contract_addr = "addr9995".to_string();
    let new_manual_ltv = Decimal256::from_str("0.1").unwrap();
    let new_fee_rate = Decimal256::from_str("0.77").unwrap();
    let new_tax_rate = Decimal256::from_str("0.9798").unwrap();

    let change_config_msg = ExecuteMsg::GovernanceMsg {
        governance_msg: GovernanceMsg::UpdateConfig {
            governance_contract_addr: Some(new_governance_contract_addr.clone()),
            nasset_token_rewards_contract_addr: Some(
                new_nasset_token_rewards_contract_addr.clone(),
            ),
            community_pool_contract_addr: Some(new_community_pool_contract_addr.clone()),
            basset_vault_strategy_contract_addr: Some(
                new_basset_vault_strategy_contract_addr.clone(),
            ),
            manual_ltv: Some(new_manual_ltv),
            fee_rate: Some(new_fee_rate),
            tax_rate: Some(new_tax_rate),
        },
    };

    let env = mock_env();
    let info = mock_info(GOVERNANCE_CONTRACT_ADDR, &[]);
    crate::contract::execute(sdk.deps.as_mut(), env, info, change_config_msg).unwrap();

    let config = load_config(&sdk.deps.storage).unwrap();
    assert_eq!(new_governance_contract_addr, config.governance_contract);
    assert_eq!(
        new_nasset_token_rewards_contract_addr,
        config.nasset_token_rewards_contract
    );
    assert_eq!(
        new_basset_vault_strategy_contract_addr,
        config.basset_vault_strategy_contract
    );
    assert_eq!(
        new_community_pool_contract_addr,
        config.community_pool_contract
    );
    assert_eq!(new_manual_ltv, config.manual_ltv);
    assert_eq!(new_fee_rate, config.fee_rate);
    assert_eq!(new_tax_rate, config.tax_rate);
}
