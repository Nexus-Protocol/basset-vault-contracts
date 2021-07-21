use crate::error::ContractError;
use crate::state::load_config;

use crate::tests::{
    ANCHOR_CUSTODY_BASSET_CONTRACT, ANCHOR_MARKET_CONTRACT, ANCHOR_OVERSEER_CONTRACT, ANCHOR_TOKEN,
    ANC_STABLE_SWAP_CONTRACT, ATERRA_TOKEN, BASSET_FARMER_CONFIG_CONTRACT, BASSET_TOKEN_ADDR,
    CLAIMING_REWARDS_DELAY, GOVERNANCE_CONTRACT, GOVERNANCE_STAKER_REWARDS_SHARE,
    NASSET_TOKEN_HOLDERS_REWARDS_SHARE, OVER_LOAN_BALANCE_VALUE, PSI_STABLE_SWAP_CONTRACT,
    PSI_TOKEN, STABLE_DENOM,
};
use basset_vault::basset_vault_config_holder::{ExecuteMsg, GovernanceMsg};
use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::testing::{mock_env, mock_info};

use std::str::FromStr;

#[test]
fn fail_to_change_config_if_sender_is_not_governance() {
    let mut deps = mock_dependencies(&[]);

    let msg = basset_vault::basset_vault_config_holder::InstantiateMsg {
        governance_contract_addr: GOVERNANCE_CONTRACT.to_string(),
        claiming_rewards_delay: CLAIMING_REWARDS_DELAY,
        basset_token_addr: BASSET_TOKEN_ADDR.to_string(),
        anchor_custody_basset_contract_addr: ANCHOR_CUSTODY_BASSET_CONTRACT.to_string(),
        anchor_overseer_contract_addr: ANCHOR_OVERSEER_CONTRACT.to_string(),
        anchor_token_addr: ANCHOR_TOKEN.to_string(),
        anchor_market_contract_addr: ANCHOR_MARKET_CONTRACT.to_string(),
        anc_stable_swap_contract_addr: ANC_STABLE_SWAP_CONTRACT.to_string(),
        psi_stable_swap_contract_addr: PSI_STABLE_SWAP_CONTRACT.to_string(),
        aterra_token_addr: ATERRA_TOKEN.to_string(),
        psi_token_addr: PSI_TOKEN.to_string(),
        basset_vault_strategy_contract_addr: BASSET_FARMER_CONFIG_CONTRACT.to_string(),
        stable_denom: STABLE_DENOM.to_string(),
        over_loan_balance_value: OVER_LOAN_BALANCE_VALUE.to_string(),
        nasset_token_holders_psi_rewards_share: NASSET_TOKEN_HOLDERS_REWARDS_SHARE,
        governance_contract_psi_rewards_share: GOVERNANCE_STAKER_REWARDS_SHARE,
    };

    let env = mock_env();
    let info = mock_info("addr9999", &[]);
    crate::contract::instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

    // ====================================
    // ====================================
    // ====================================

    let new_governance_contract_addr = Some("addr7777".to_string());

    let change_config_msg = ExecuteMsg::Governance {
        governance_msg: GovernanceMsg::UpdateConfig {
            governance_contract_addr: new_governance_contract_addr,
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
    let info = mock_info("addr9999", &[]);
    let res = crate::contract::execute(deps.as_mut(), env, info, change_config_msg);
    assert!(res.is_err());
    assert_eq!(ContractError::Unauthorized, res.err().unwrap());
}

#[test]
fn success_to_change_config_if_sender_governance() {
    let mut deps = mock_dependencies(&[]);

    let msg = basset_vault::basset_vault_config_holder::InstantiateMsg {
        governance_contract_addr: GOVERNANCE_CONTRACT.to_string(),
        claiming_rewards_delay: CLAIMING_REWARDS_DELAY,
        basset_token_addr: BASSET_TOKEN_ADDR.to_string(),
        anchor_custody_basset_contract_addr: ANCHOR_CUSTODY_BASSET_CONTRACT.to_string(),
        anchor_overseer_contract_addr: ANCHOR_OVERSEER_CONTRACT.to_string(),
        anchor_token_addr: ANCHOR_TOKEN.to_string(),
        anchor_market_contract_addr: ANCHOR_MARKET_CONTRACT.to_string(),
        anc_stable_swap_contract_addr: ANC_STABLE_SWAP_CONTRACT.to_string(),
        psi_stable_swap_contract_addr: PSI_STABLE_SWAP_CONTRACT.to_string(),
        aterra_token_addr: ATERRA_TOKEN.to_string(),
        psi_token_addr: PSI_TOKEN.to_string(),
        basset_vault_strategy_contract_addr: BASSET_FARMER_CONFIG_CONTRACT.to_string(),
        stable_denom: STABLE_DENOM.to_string(),
        over_loan_balance_value: OVER_LOAN_BALANCE_VALUE.to_string(),
        nasset_token_holders_psi_rewards_share: NASSET_TOKEN_HOLDERS_REWARDS_SHARE,
        governance_contract_psi_rewards_share: GOVERNANCE_STAKER_REWARDS_SHARE,
    };

    let env = mock_env();
    let info = mock_info("addr9999", &[]);
    crate::contract::instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // ====================================
    // ====================================
    // ====================================

    let new_governance_contract_addr = "addr7777".to_string();
    let new_anchor_overseer_contract_addr = "addr7001".to_string();
    let new_anchor_market_contract_addr = "addr7002".to_string();
    let new_anchor_custody_basset_contract_addr = "addr7003".to_string();
    let new_anc_stable_swap_contract_addr = "addr7004".to_string();
    let new_psi_stable_swap_contract_addr = "addr7005".to_string();
    let new_basset_vault_strategy_contract_addr = "addr7006".to_string();
    let new_claiming_rewards_delay = 400u64;
    let new_over_loan_balance_value = "1.5".to_string();

    let change_config_msg = ExecuteMsg::Governance {
        governance_msg: GovernanceMsg::UpdateConfig {
            governance_contract_addr: Some(new_governance_contract_addr.clone()),
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
            over_loan_balance_value: Some(new_over_loan_balance_value.clone()),
        },
    };

    let env = mock_env();
    let info = mock_info(GOVERNANCE_CONTRACT, &[]);
    crate::contract::execute(deps.as_mut(), env.clone(), info.clone(), change_config_msg).unwrap();

    let config = load_config(&deps.storage).unwrap();
    assert_eq!(new_governance_contract_addr, config.governance_contract);
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
    assert_eq!(
        Decimal256::from_str(&new_over_loan_balance_value).unwrap(),
        config.over_loan_balance_value
    );
}
