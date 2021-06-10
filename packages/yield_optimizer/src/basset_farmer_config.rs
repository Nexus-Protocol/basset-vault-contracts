use std::collections::VecDeque;

use cosmwasm_std::{Addr, Decimal};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub governance_contract_addr: String,
    pub oracle_addr: String,
    pub basset_token_addr: String,
    pub stable_denom: String,
    pub borrow_ltv_max: Decimal256,
    pub borrow_ltv_min: Decimal256,
    pub borrow_ltv_aim: Decimal256,
    pub basset_max_ltv: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    GovernanceMsg { overseer_msg: GovernanceMsg },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceMsg {
    UpdateConfig {
        oracle_addr: Option<String>,
        basset_token_addr: Option<String>,
        stable_denom: Option<String>,
        borrow_ltv_max: Option<Decimal256>,
        borrow_ltv_min: Option<Decimal256>,
        borrow_ltv_aim: Option<Decimal256>,
        basset_max_ltv: Option<Decimal256>,
    },
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    BorrowerAction {
        borrowed_amount: Uint256,
        locked_basset_amount: Uint256,
    },
}

//TODO: update, cause Config struct changed
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub governance_contract_addr: Addr,
    pub oracle_addr: Addr,
    pub basset_token_addr: Addr,
    pub stable_denom: String,
    pub borrow_ltv_max: Decimal256,
    pub borrow_ltv_min: Decimal256,
    pub borrow_ltv_aim: Decimal256,
    pub basset_max_ltv: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum BorrowerActionResponse {
    Nothing {},
    Borrow { amount: Uint256 },
    Repay { amount: Uint256 },
}

impl BorrowerActionResponse {
    pub fn repay(amount: Uint256) -> Self {
        BorrowerActionResponse::Repay { amount }
    }

    pub fn borrow(amount: Uint256) -> Self {
        BorrowerActionResponse::Borrow { amount }
    }

    pub fn nothing() -> Self {
        BorrowerActionResponse::Nothing {}
    }
}
