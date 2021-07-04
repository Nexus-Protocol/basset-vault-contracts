use crate::state::{load_config, Config, RewardShare};
use crate::tests::sdk::{
    GOVERNANCE_CONTRACT_ADDR, NASSET_TOKEN_REWARDS_CONTRACT_ADDR, PSI_TOKEN_ADDR,
};

use super::sdk::Sdk;
use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::Addr;

#[test]
fn proper_initialization() {
    let mut sdk = Sdk::init();

    let config: Config = load_config(&sdk.deps.storage).unwrap();
    assert_eq!(PSI_TOKEN_ADDR, config.psi_token);
    assert_eq!(GOVERNANCE_CONTRACT_ADDR, config.governance_contract);
    let distribution = config.rewards_distribution.distribution();
    assert_eq!(
        *distribution,
        vec![
            RewardShare {
                recipient: Addr::unchecked(NASSET_TOKEN_REWARDS_CONTRACT_ADDR),
                share: Decimal256::percent(70)
            },
            RewardShare {
                recipient: Addr::unchecked(GOVERNANCE_CONTRACT_ADDR),
                share: Decimal256::percent(30)
            }
        ]
    );
}
