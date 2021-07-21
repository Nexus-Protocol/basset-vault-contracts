use basset_vault::basset_vault_config_holder::Config;
use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::testing::{mock_env, mock_info};

use crate::state::load_config;
use crate::tests::{
    ANCHOR_CUSTODY_BASSET_CONTRACT, ANCHOR_MARKET_CONTRACT, ANCHOR_OVERSEER_CONTRACT, ANCHOR_TOKEN,
    ANC_STABLE_SWAP_CONTRACT, ATERRA_TOKEN, BASSET_FARMER_CONFIG_CONTRACT, BASSET_TOKEN_ADDR,
    CLAIMING_REWARDS_DELAY, GOVERNANCE_CONTRACT, GOVERNANCE_STAKER_REWARDS_SHARE,
    NASSET_TOKEN_HOLDERS_REWARDS_SHARE, OVER_LOAN_BALANCE_VALUE, PSI_STABLE_SWAP_CONTRACT,
    PSI_TOKEN, STABLE_DENOM,
};
use std::str::FromStr;

#[test]
fn proper_initialization() {
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

    let config: Config = load_config(&deps.storage).unwrap();
    assert_eq!(GOVERNANCE_CONTRACT, config.governance_contract);
    assert_eq!(CLAIMING_REWARDS_DELAY, config.claiming_rewards_delay);
    assert_eq!(BASSET_TOKEN_ADDR, config.basset_token);
    assert_eq!(
        ANCHOR_CUSTODY_BASSET_CONTRACT,
        config.anchor_custody_basset_contract
    );
    assert_eq!(ANCHOR_OVERSEER_CONTRACT, config.anchor_overseer_contract);
    assert_eq!(ANCHOR_TOKEN, config.anchor_token);
    assert_eq!(ANCHOR_MARKET_CONTRACT, config.anchor_market_contract);
    assert_eq!(ANC_STABLE_SWAP_CONTRACT, config.anc_stable_swap_contract);
    assert_eq!(PSI_STABLE_SWAP_CONTRACT, config.psi_stable_swap_contract);
    assert_eq!(ATERRA_TOKEN, config.aterra_token);
    assert_eq!(PSI_TOKEN, config.psi_token);
    assert_eq!(
        BASSET_FARMER_CONFIG_CONTRACT,
        config.basset_vault_strategy_contract
    );
    assert_eq!(STABLE_DENOM, config.stable_denom);
    assert_eq!(
        Decimal256::from_str(OVER_LOAN_BALANCE_VALUE).unwrap(),
        config.over_loan_balance_value
    );
}
