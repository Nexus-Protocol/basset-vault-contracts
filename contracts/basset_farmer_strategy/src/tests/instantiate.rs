use crate::state::{load_config, Config};

use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::testing::{mock_env, mock_info};
use std::str::FromStr;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = yield_optimizer::basset_farmer_strategy::InstantiateMsg {
        governance_contract_addr: "addr0000".to_string(),
        oracle_contract_addr: "addr0001".to_string(),
        basset_token_addr: "addr0002".to_string(),
        stable_denom: "uust".to_string(),
        borrow_ltv_max: Decimal256::from_str("0.85").unwrap(),
        borrow_ltv_min: Decimal256::from_str("0.75").unwrap(),
        borrow_ltv_aim: Decimal256::from_str("0.8").unwrap(),
        basset_max_ltv: Decimal256::from_str("0.5").unwrap(),
        buffer_part: Decimal256::from_str("0.018").unwrap(),
        price_timeframe: 60,
    };

    let env = mock_env();
    let info = mock_info("addr0010", &[]);
    crate::contract::instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();

    // it worked, let's query the state
    let config: Config = load_config(&deps.storage).unwrap();
    assert_eq!(msg.governance_contract_addr, config.governance_contract);
    assert_eq!(msg.oracle_contract_addr, config.oracle_contract);
    assert_eq!(msg.basset_token_addr, config.basset_token);
    assert_eq!(msg.stable_denom, config.stable_denom);
    assert_eq!(msg.borrow_ltv_max, config.get_borrow_ltv_max());
    assert_eq!(msg.borrow_ltv_min, config.get_borrow_ltv_min());
    assert_eq!(msg.borrow_ltv_aim, config.get_borrow_ltv_aim());
    assert_eq!(msg.basset_max_ltv, config.get_basset_max_ltv());
    assert_eq!(msg.buffer_part, config.get_buffer_part());
    assert_eq!(msg.price_timeframe, config.price_timeframe);
}
