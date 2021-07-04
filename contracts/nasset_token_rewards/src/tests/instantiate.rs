use crate::state::{load_config, load_state, Config, State};
use crate::tests::sdk::{GOVERNANCE_CONTRACT_ADDR, NASSET_TOKEN_ADDR, PSI_TOKEN_ADDR};

use super::sdk::Sdk;
use cosmwasm_std::{Decimal, Uint128};

#[test]
fn proper_initialization() {
    let sdk = Sdk::init();

    // it worked, let's query the state
    let config: Config = load_config(&sdk.deps.storage).unwrap();
    assert_eq!(PSI_TOKEN_ADDR, config.psi_token);
    assert_eq!(NASSET_TOKEN_ADDR, config.nasset_token);
    assert_eq!(GOVERNANCE_CONTRACT_ADDR, config.governance_contract);

    let state: State = load_state(&sdk.deps.storage).unwrap();
    assert_eq!(Decimal::zero(), state.global_index);
    assert_eq!(Uint128::zero(), state.total_balance);
    assert_eq!(Uint128::zero(), state.prev_reward_balance);
}
