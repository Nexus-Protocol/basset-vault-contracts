use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub nasset_token_code_id: u64,
    pub nasset_staker_code_id: u64,
    pub psi_distributor_code_id: u64,
    pub basset_token_addr: String,
    //Luna / ETH / Sol, will be converted to nLuna, nETH, nSol
    pub collateral_token_symbol: String,
    pub governance_addr: String,
    pub anchor_token: String,
    pub anchor_market_contract: String,
    pub anchor_overseer_contract: String,
    pub anchor_custody_basset_contract: String,
    pub anc_stable_swap_contract: String,
    pub psi_stable_swap_contract: String,
    pub aterra_token: String,
    pub psi_token: String,
    pub basset_farmer_config_contract: String,
    pub stable_denom: String,
    pub claiming_rewards_delay: u64,
    pub over_loan_balance_value: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Anyone { anyone_msg: AnyoneMsg },
    Token { token_msg: TokenMsg },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AnyoneMsg {
    /// Update the global index
    UpdateGlobalIndex {},

    /// return the accrued reward in uusd to the user.
    ClaimRewards { recipient: Option<String> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TokenMsg {
    /// Increase user staking balance
    /// Withdraw rewards to pending rewards
    /// Set current reward index to global index
    IncreaseBalance { address: String, amount: Uint128 },
    /// Decrease user staking balance
    /// Withdraw rewards to pending rewards
    /// Set current reward index to global index
    DecreaseBalance { address: String, amount: Uint128 },
}
