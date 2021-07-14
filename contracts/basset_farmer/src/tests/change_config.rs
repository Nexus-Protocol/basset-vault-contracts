use crate::error::ContractError;
use crate::state::load_config;
use crate::tests::sdk::GOVERNANCE_CONTRACT;

use super::sdk::Sdk;
use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::testing::{mock_env, mock_info};
use std::str::FromStr;
use yield_optimizer::basset_farmer::{ExecuteMsg, GovernanceMsg};

#[test]
fn fail_to_change_config_if_sender_is_not_governance() {
    let mut sdk = Sdk::init();

    let change_config_msg = ExecuteMsg::Governance {
        governance_msg: GovernanceMsg::UpdateConfig {
            governance_contract_addr: Some("addr9999".to_string()),
            psi_distributor_addr: Some("addr9998".to_string()),
            anchor_token_addr: Some("addr9997".to_string()),
            anchor_overseer_contract_addr: Some("addr9996".to_string()),
            anchor_market_contract_addr: Some("addr9995".to_string()),
            anchor_custody_basset_contract_addr: Some("addr9994".to_string()),
            anc_stable_swap_contract_addr: Some("addr9993".to_string()),
            psi_stable_swap_contract_addr: Some("addr9992".to_string()),
            nasset_token_addr: Some("addr9991".to_string()),
            basset_token_addr: Some("addr9990".to_string()),
            aterra_token_addr: Some("addr9989".to_string()),
            psi_token_addr: Some("addr9988".to_string()),
            basset_farmer_strategy_contract_addr: Some("addr9987".to_string()),
            stable_denom: Some("ukrt".to_string()),
            claiming_rewards_delay: Some(1000),
            over_loan_balance_value: Some("1.10".to_string()),
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

    let new_governance_contract_addr = "addr9999".to_string();
    let new_psi_distributor_addr = "addr9998".to_string();
    let new_anchor_token_addr = "addr9997".to_string();
    let new_anchor_overseer_contract_addr = "addr9996".to_string();
    let new_anchor_market_contract_addr = "addr9995".to_string();
    let new_anchor_custody_basset_contract_addr = "addr9994".to_string();
    let new_anc_stable_swap_contract_addr = "addr9993".to_string();
    let new_psi_stable_swap_contract_addr = "addr9992".to_string();
    let new_nasset_token_addr = "addr9991".to_string();
    let new_basset_token_addr = "addr9990".to_string();
    let new_aterra_token_addr = "addr9989".to_string();
    let new_psi_token_addr = "addr9988".to_string();
    let new_basset_farmer_strategy_contract_addr = "addr9987".to_string();
    let new_stable_denom = "ukrt".to_string();
    let new_claiming_rewards_delay = 1000;
    let new_over_loan_balance_value = "1.10".to_string();

    let change_config_msg = ExecuteMsg::Governance {
        governance_msg: GovernanceMsg::UpdateConfig {
            governance_contract_addr: Some(new_governance_contract_addr.clone()),
            psi_distributor_addr: Some(new_psi_distributor_addr.clone()),
            anchor_token_addr: Some(new_anchor_token_addr.clone()),
            anchor_overseer_contract_addr: Some(new_anchor_overseer_contract_addr.clone()),
            anchor_market_contract_addr: Some(new_anchor_market_contract_addr.clone()),
            anchor_custody_basset_contract_addr: Some(
                new_anchor_custody_basset_contract_addr.clone(),
            ),
            anc_stable_swap_contract_addr: Some(new_anc_stable_swap_contract_addr.clone()),
            psi_stable_swap_contract_addr: Some(new_psi_stable_swap_contract_addr.clone()),
            nasset_token_addr: Some(new_nasset_token_addr.clone()),
            basset_token_addr: Some(new_basset_token_addr.clone()),
            aterra_token_addr: Some(new_aterra_token_addr.clone()),
            psi_token_addr: Some(new_psi_token_addr.clone()),
            basset_farmer_strategy_contract_addr: Some(
                new_basset_farmer_strategy_contract_addr.clone(),
            ),
            stable_denom: Some(new_stable_denom.clone()),
            claiming_rewards_delay: Some(new_claiming_rewards_delay),
            over_loan_balance_value: Some(new_over_loan_balance_value.clone()),
        },
    };

    let env = mock_env();
    let info = mock_info(GOVERNANCE_CONTRACT, &[]);
    crate::contract::execute(sdk.deps.as_mut(), env, info, change_config_msg).unwrap();

    let config = load_config(&sdk.deps.storage).unwrap();
    assert_eq!(new_governance_contract_addr, config.governance_contract);
    assert_eq!(new_psi_distributor_addr, config.psi_distributor);
    assert_eq!(new_anchor_token_addr, config.anchor_token);
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
    assert_eq!(new_nasset_token_addr, config.nasset_token);
    assert_eq!(new_basset_token_addr, config.basset_token);
    assert_eq!(new_aterra_token_addr, config.aterra_token);
    assert_eq!(new_psi_token_addr, config.psi_token);
    assert_eq!(
        new_basset_farmer_strategy_contract_addr,
        config.basset_farmer_strategy_contract
    );
    assert_eq!(new_stable_denom, config.stable_denom);
    assert_eq!(new_claiming_rewards_delay, config.claiming_rewards_delay);
    assert_eq!(
        Decimal256::from_str(&new_over_loan_balance_value).unwrap(),
        config.over_loan_balance_value
    );
}
