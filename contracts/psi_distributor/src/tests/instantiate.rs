use std::str::FromStr;

use crate::state::{load_config, Config};
use crate::tests::sdk::{
    BASSET_VAULT_STRATEGY_CONTRACT_ADDR, COMMUNITY_POOL_CONTRACT_ADDR, FEE_RATE,
    GOVERNANCE_CONTRACT_ADDR, MANUAL_LTV, NASSET_TOKEN_REWARDS_CONTRACT_ADDR, PSI_TOKEN_ADDR,
    TAX_RATE,
};

use super::sdk::Sdk;
use cosmwasm_bignumber::Decimal256;

#[test]
fn proper_initialization() {
    let sdk = Sdk::init();

    let config: Config = load_config(&sdk.deps.storage).unwrap();
    assert_eq!(PSI_TOKEN_ADDR, config.psi_token);
    assert_eq!(GOVERNANCE_CONTRACT_ADDR, config.governance_contract);
    assert_eq!(
        NASSET_TOKEN_REWARDS_CONTRACT_ADDR,
        config.nasset_token_rewards_contract
    );
    assert_eq!(COMMUNITY_POOL_CONTRACT_ADDR, config.community_pool_contract);
    assert_eq!(
        BASSET_VAULT_STRATEGY_CONTRACT_ADDR,
        config.basset_vault_strategy_contract
    );
    assert_eq!(Decimal256::from_str(MANUAL_LTV).unwrap(), config.manual_ltv);
    assert_eq!(Decimal256::from_str(FEE_RATE).unwrap(), config.fee_rate);
    assert_eq!(Decimal256::from_str(TAX_RATE).unwrap(), config.tax_rate);
}
