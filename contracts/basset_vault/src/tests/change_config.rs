use std::str::FromStr;

use crate::state::load_config;
use crate::tests::sdk::GOVERNANCE_CONTRACT;

use super::sdk::Sdk;
use basset_vault::basset_vault::{ExecuteMsg, GovernanceMsg};
use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::StdError;

#[test]
fn fail_to_change_config_if_sender_is_not_governance() {
    let mut sdk = Sdk::init();

    let change_config_msg = ExecuteMsg::Governance {
        governance_msg: GovernanceMsg::UpdateConfig {
            psi_distributor_addr: Some("addr9998".to_string()),
            anchor_overseer_contract_addr: None,
            anchor_market_contract_addr: None,
            anchor_custody_basset_contract_addr: None,
            anc_stable_swap_contract_addr: None,
            psi_stable_swap_contract_addr: None,
            basset_vault_strategy_contract_addr: None,
            claiming_rewards_delay: None,
            over_loan_balance_value: None,
        },
    };

    let env = mock_env();
    let info = mock_info("addr0010", &[]);
    let res = crate::contract::execute(sdk.deps.as_mut(), env, info, change_config_msg);
    assert!(res.is_err());
    if let StdError::GenericErr { msg, .. } = res.err().unwrap() {
        assert_eq!("unauthorized", msg);
    } else {
        panic!("wrong error");
    }
}

#[test]
fn success_to_change_config_if_sender_governance() {
    let mut sdk = Sdk::init();

    let new_psi_distributor_addr = "addr9994".to_string();
    let new_anchor_overseer_contract_addr = "addr9993".to_string();
    let new_anchor_market_contract_addr = "addr9992".to_string();
    let new_anchor_custody_basset_contract_addr = "addr9991".to_string();
    let new_anc_stable_swap_contract_addr = "addr9990".to_string();
    let new_psi_stable_swap_contract_addr = "addr9989".to_string();
    let new_basset_vault_strategy_contract_addr = "addr9988".to_string();
    let new_claiming_rewards_delay = 555;
    let new_over_loan_balance_value = Decimal256::from_str("1.88").unwrap();

    let change_config_msg = ExecuteMsg::Governance {
        governance_msg: GovernanceMsg::UpdateConfig {
            psi_distributor_addr: Some(new_psi_distributor_addr.clone()),
            anchor_overseer_contract_addr: Some(new_anchor_overseer_contract_addr.clone()),
            anchor_market_contract_addr: Some(new_anchor_market_contract_addr.clone()),
            anchor_custody_basset_contract_addr: Some(
                new_anchor_custody_basset_contract_addr.clone(),
            ),
            anc_stable_swap_contract_addr: Some(new_anc_stable_swap_contract_addr.clone()),
            psi_stable_swap_contract_addr: Some(new_psi_stable_swap_contract_addr.clone()),
            basset_vault_strategy_contract_addr: Some(
                new_basset_vault_strategy_contract_addr.clone(),
            ),
            claiming_rewards_delay: Some(new_claiming_rewards_delay),
            over_loan_balance_value: Some(new_over_loan_balance_value),
        },
    };

    let env = mock_env();
    let info = mock_info(GOVERNANCE_CONTRACT, &[]);
    crate::contract::execute(sdk.deps.as_mut(), env, info, change_config_msg).unwrap();

    let config = load_config(&sdk.deps.storage).unwrap();
    assert_eq!(new_psi_distributor_addr, config.psi_distributor);
    assert_eq!(
        new_anchor_overseer_contract_addr,
        config.anchor_overseer_contract
    );
    assert_eq!(
        new_anchor_market_contract_addr,
        config.anchor_market_contract
    );
    assert_eq!(
        new_anchor_custody_basset_contract_addr,
        config.anchor_custody_basset_contract
    );
    assert_eq!(
        new_anc_stable_swap_contract_addr,
        config.anc_stable_swap_contract
    );
    assert_eq!(
        new_psi_stable_swap_contract_addr,
        config.psi_stable_swap_contract
    );
    assert_eq!(
        new_basset_vault_strategy_contract_addr,
        config.basset_vault_strategy_contract
    );
    assert_eq!(new_claiming_rewards_delay, config.claiming_rewards_delay);
    assert_eq!(new_over_loan_balance_value, config.over_loan_balance_value);
}
