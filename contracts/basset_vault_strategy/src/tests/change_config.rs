use crate::error::ContractError;
use crate::state::load_config;

use basset_vault::basset_vault_strategy::{ExecuteMsg, GovernanceMsg};
use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::testing::{mock_env, mock_info};
use std::str::FromStr;

#[test]
fn fail_to_change_config_if_sender_is_not_governance() {
    let mut deps = mock_dependencies(&[]);

    let msg = basset_vault::basset_vault_strategy::InstantiateMsg {
        governance_contract_addr: "addr0000".to_string(),
        oracle_contract_addr: "addr0001".to_string(),
        anchor_market_addr: "addr0002".to_string(),
        anchor_interest_model_addr: "addr0003".to_string(),
        anchor_overseer_addr: "addr0004".to_string(),
        anc_ust_swap_addr: "addr0005".to_string(),
        basset_token_addr: "addr0006".to_string(),
        stable_denom: "uust".to_string(),
        borrow_ltv_max: Decimal256::from_str("0.85").unwrap(),
        borrow_ltv_min: Decimal256::from_str("0.75").unwrap(),
        borrow_ltv_aim: Decimal256::from_str("0.8").unwrap(),
        basset_max_ltv: Decimal256::from_str("0.5").unwrap(),
        buffer_part: Decimal256::from_str("0.018").unwrap(),
        price_timeframe: 60,
    };

    {
        let env = mock_env();
        let info = mock_info("addr0010", &[]);
        crate::contract::instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();
    }

    // ====================================
    // ====================================
    // ====================================

    let new_oracle_addr = Some("addr9999".to_string());
    let new_basset_token_addr = Some("addr9998".to_string());
    let new_stable_denom = Some("addr9997".to_string());
    let new_anchor_market_addr = Some("addr9996".to_string());
    let new_anchor_interest_model_addr = Some("addr9995".to_string());
    let new_overseer_addr = Some("addr9994".to_string());
    let new_anc_ust_swap_addr = Some("addr9992".to_string());
    let new_borrow_ltv_max = Some(Decimal256::from_str("0.6").unwrap());
    let new_borrow_ltv_min = Some(Decimal256::from_str("0.4").unwrap());
    let new_borrow_ltv_aim = Some(Decimal256::from_str("0.5").unwrap());
    let new_basset_max_ltv = Some(Decimal256::from_str("0.7").unwrap());
    let new_buffer_part = Some(Decimal256::from_str("0.99").unwrap());
    let new_price_timeframe = Some(100);

    let change_config_msg = ExecuteMsg::Governance {
        governance_msg: GovernanceMsg::UpdateConfig {
            oracle_addr: new_oracle_addr,
            basset_token_addr: new_basset_token_addr,
            stable_denom: new_stable_denom,
            borrow_ltv_max: new_borrow_ltv_max,
            borrow_ltv_min: new_borrow_ltv_min,
            borrow_ltv_aim: new_borrow_ltv_aim,
            basset_max_ltv: new_basset_max_ltv,
            buffer_part: new_buffer_part,
            price_timeframe: new_price_timeframe,
            anchor_market_addr: new_anchor_market_addr,
            anchor_interest_model_addr: new_anchor_interest_model_addr,
            anchor_overseer_addr: new_overseer_addr,
            anc_ust_swap_addr: new_anc_ust_swap_addr,
        },
    };

    let env = mock_env();
    let info = mock_info("addr0010", &[]);
    let res = crate::contract::execute(deps.as_mut(), env, info, change_config_msg);
    assert!(res.is_err());
    assert_eq!(ContractError::Unauthorized, res.err().unwrap());
}

#[test]
fn success_to_change_config_if_sender_governance() {
    let mut deps = mock_dependencies(&[]);
    let governance_contract_addr = "addr0000".to_string();

    let msg = basset_vault::basset_vault_strategy::InstantiateMsg {
        governance_contract_addr: governance_contract_addr.clone(),
        oracle_contract_addr: "addr0001".to_string(),
        anchor_market_addr: "addr0002".to_string(),
        anchor_interest_model_addr: "addr0003".to_string(),
        anchor_overseer_addr: "addr0004".to_string(),
        anc_ust_swap_addr: "addr0006".to_string(),
        basset_token_addr: "addr0007".to_string(),
        stable_denom: "uust".to_string(),
        borrow_ltv_max: Decimal256::from_str("0.85").unwrap(),
        borrow_ltv_min: Decimal256::from_str("0.75").unwrap(),
        borrow_ltv_aim: Decimal256::from_str("0.8").unwrap(),
        basset_max_ltv: Decimal256::from_str("0.5").unwrap(),
        buffer_part: Decimal256::from_str("0.018").unwrap(),
        price_timeframe: 60,
    };

    {
        let env = mock_env();
        let info = mock_info("addr0010", &[]);
        crate::contract::instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();
    }

    // ====================================
    // ====================================
    // ====================================

    let new_oracle_addr = "addr9999".to_string();
    let new_basset_token_addr = "addr9998".to_string();
    let new_stable_denom = "addr9997".to_string();
    let new_anchor_market_addr = "addr9996".to_string();
    let new_anchor_interest_model_addr = "addr9995".to_string();
    let new_overseer_addr = "addr9994".to_string();
    let new_anchor_token_addr = "addr9993".to_string();
    let new_anc_ust_swap_addr = "addr9992".to_string();
    let new_borrow_ltv_max = Decimal256::from_str("0.6").unwrap();
    let new_borrow_ltv_min = Decimal256::from_str("0.4").unwrap();
    let new_borrow_ltv_aim = Decimal256::from_str("0.5").unwrap();
    let new_basset_max_ltv = Decimal256::from_str("0.7").unwrap();
    let new_buffer_part = Decimal256::from_str("0.99").unwrap();
    let new_price_timeframe = 100;

    let change_config_msg = ExecuteMsg::Governance {
        governance_msg: GovernanceMsg::UpdateConfig {
            oracle_addr: Some(new_oracle_addr.clone()),
            basset_token_addr: Some(new_basset_token_addr.clone()),
            stable_denom: Some(new_stable_denom.clone()),
            borrow_ltv_max: Some(new_borrow_ltv_max.clone()),
            borrow_ltv_min: Some(new_borrow_ltv_min.clone()),
            borrow_ltv_aim: Some(new_borrow_ltv_aim.clone()),
            basset_max_ltv: Some(new_basset_max_ltv.clone()),
            buffer_part: Some(new_buffer_part.clone()),
            price_timeframe: Some(new_price_timeframe),
            anchor_market_addr: Some(new_anchor_market_addr.clone()),
            anchor_interest_model_addr: Some(new_anchor_interest_model_addr.clone()),
            anchor_overseer_addr: Some(new_overseer_addr.clone()),
            anc_ust_swap_addr: Some(new_anc_ust_swap_addr.clone()),
        },
    };

    let env = mock_env();
    let info = mock_info(&governance_contract_addr, &[]);
    crate::contract::execute(deps.as_mut(), env, info, change_config_msg).unwrap();

    let config = load_config(&deps.storage).unwrap();
    assert_eq!(new_oracle_addr, config.oracle_contract);
    assert_eq!(new_basset_token_addr, config.basset_token);
    assert_eq!(new_stable_denom, config.stable_denom);
    assert_eq!(new_borrow_ltv_max, config.get_borrow_ltv_max());
    assert_eq!(new_borrow_ltv_min, config.get_borrow_ltv_min());
    assert_eq!(new_borrow_ltv_aim, config.get_borrow_ltv_aim());
    assert_eq!(new_basset_max_ltv, config.get_basset_max_ltv());
    assert_eq!(new_buffer_part, config.get_buffer_part());
    assert_eq!(new_price_timeframe, config.price_timeframe);
    assert_eq!(new_anchor_market_addr, config.anchor_market_contract);
    assert_eq!(new_anchor_interest_model_addr, config.anchor_interest_model_contract);
    assert_eq!(new_overseer_addr, config.anchor_overseer_contract);
    assert_eq!(new_anc_ust_swap_addr, config.anc_ust_swap_contract);
}
