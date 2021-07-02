use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw20::{Cw20Coin, MinterResponse};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Vec<Cw20Coin>,
    pub mint: Option<MinterResponse>,
    pub config_holder_contract: String,
}
