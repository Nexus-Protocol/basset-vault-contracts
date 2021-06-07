use std::collections::VecDeque;

use cosmwasm_std::{Addr, Decimal};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub governance_contract_addr: String,
    pub borrow_ration_aim: Decimal,
    pub borrow_ration_upper_gap: Decimal,
    pub borrow_ration_bottom_gap: Decimal,
    pub stable_denom: String,
    pub oracle_addr: String,
    pub basset_token_addr: String,
    pub price_timeframe_millis: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdatePrice {},
    GovernanceMsg { overseer_msg: GovernanceMsg },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceMsg {
    UpdateConfig {
        borrow_ration_aim: Option<Decimal>,
        borrow_ration_upper_gap: Option<Decimal>,
        borrow_ration_bottom_gap: Option<Decimal>,
        oracle_addr: Option<String>,
        basset_token_addr: Option<String>,
        stable_denom: Option<String>,
    },
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    State {},
    BorrowLimits {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub borrow_ration_aim: Decimal,
    pub borrow_ration_upper_gap: Decimal,
    pub borrow_ration_bottom_gap: Decimal,
    pub oracle_addr: String,
    pub basset_token_addr: String,
    pub stable_denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub prices: VecDeque<Decimal256>,
    pub price_last_update_time: u64,

    // 1. average price value
    // 2. std_dev from prices
    // 3. std_dev / avg_price * 100
    pub last_std_dev_from_average_price: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BorrowLimitsResponse {
    pub max_ratio: Decimal,
    pub aim_ratio: Decimal,
    pub min_ratio: Decimal,
}
