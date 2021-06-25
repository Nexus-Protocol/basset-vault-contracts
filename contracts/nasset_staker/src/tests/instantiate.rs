use crate::state::{load_config, load_state, Config};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::testing::{mock_env, mock_info};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);
    let nasset_token = "addr0001".to_string();

    let msg = yield_optimizer::nasset_staker::InstantiateMsg {
        nasset_token: nasset_token.clone(),
    };

    let env = mock_env();
    let info = mock_info("addr0010", &[]);
    let _ = crate::contract::instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();

    // it worked, let's query the state
    let config: Config = load_config(&deps.storage).unwrap();
    assert_eq!(nasset_token, config.nasset_token);

    let state = load_state(&deps.storage).unwrap();
    assert_eq!(state.last_reward_amount, Uint256::zero());
    assert_eq!(state.global_reward_index, Decimal256::zero());
    assert_eq!(state.total_staked_amount, Uint256::zero());
}
