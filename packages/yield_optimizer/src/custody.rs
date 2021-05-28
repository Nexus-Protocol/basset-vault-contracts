use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{Addr, HumanAddr};
use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    /// CW20 token receiver
    Receive(Cw20ReceiveMsg),

    ////////////////////
    /// Overseer operations
    ////////////////////

    /// Update config
    UpdateConfig {},
    /// Make specified amount of tokens unspendable
    LockTokens { depositor: Addr, amount: Uint256 },
    /// Make specified amount of collateral tokens spendable
    UnlockTokens { depositor: Addr, amount: Uint256 },

    ////////////////////
    /// User operations
    ////////////////////

    /// Withdraw spendable collateral token.
    /// If the amount is not given,
    /// return all spendable collateral
    WithdrawCollateral { amount: Option<Uint256> },
}
