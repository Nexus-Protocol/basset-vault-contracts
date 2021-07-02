use crate::state::{load_config, load_state, Config, State};

use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{Decimal, Uint128};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = yield_optimizer::nasset_token_rewards::InstantiateMsg {
        psi_token_addr: "addr0000".to_string(),
        nasset_token_addr: "addr0001".to_string(),
        governance_contract_addr: "addr0002".to_string(),
    };

    let env = mock_env();
    let info = mock_info("addr0010", &[]);
    crate::contract::instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();

    // it worked, let's query the state
    let config: Config = load_config(&deps.storage).unwrap();
    assert_eq!(msg.psi_token_addr, config.psi_token);
    assert_eq!(msg.nasset_token_addr, config.nasset_token);
    assert_eq!(msg.governance_contract_addr, config.governance_contract);

    let state: State = load_state(&deps.storage).unwrap();
    assert_eq!(Decimal::zero(), state.global_index);
    assert_eq!(Uint128::zero(), state.total_balance);
    assert_eq!(Uint128::zero(), state.prev_reward_balance);
}
