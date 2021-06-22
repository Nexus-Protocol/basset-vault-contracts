use crate::state::{load_config, load_state, Config};

use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::testing::{mock_env, mock_info};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);
    let casset_token = "addr0001".to_string();
    let aterra_token = "addr0002".to_string();
    let stable_denom = "uust".to_string();
    let basset_farmer_contract = "addr0003".to_string();
    let anchor_market_contract = "addr0004".to_string();

    let msg = yield_optimizer::casset_staking::InstantiateMsg {
        casset_token: casset_token.clone(),
        aterra_token: aterra_token.clone(),
        stable_denom: stable_denom.clone(),
        basset_farmer_contract: basset_farmer_contract.clone(),
        anchor_market_contract: anchor_market_contract.clone(),
    };

    let env = mock_env();
    let info = mock_info("addr0010", &[]);
    let _ = crate::contract::instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();

    // it worked, let's query the state
    let config: Config = load_config(&deps.storage).unwrap();
    assert_eq!(casset_token, config.casset_token);
    assert_eq!(aterra_token, config.aterra_token);
    assert_eq!(stable_denom, config.stable_denom);
    assert_eq!(basset_farmer_contract, config.basset_farmer_contract);
    assert_eq!(anchor_market_contract, config.anchor_market_contract);

    let state = load_state(&deps.storage).unwrap();
    assert_eq!(state.last_reward_amount, Decimal256::zero());
    assert_eq!(state.global_reward_index, Decimal256::zero());
    assert_eq!(state.last_reward_updated, 0);
}
