use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cw20::Cw20ReceiveMsg;

use terraswap::asset::Asset;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UserMsg { user_msg: UserMsg },
    MaintainerMsg { mantainer_msg: MaintainerMsg },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UserMsg {
    Deposit { asset: Asset },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MaintainerMsg {
    /// Create new custody contract for the given collateral token
    Whitelist {
        name: String,
        collateral_token: String,
        strategy_contract: String,
    },
    /// Update registered whitelist info
    UpdateWhitelist {
        collateral_token: String,         // bAsset token contract
        custody_contract: Option<String>, // bAsset custody contract
        max_ltv: Option<Decimal256>,      // Loan To Value ratio
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    /// Return stable coins to a user
    /// according to exchange rate
    Deposit {},
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Assets {
        depositor: String,
    },
    AllAssets {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}
