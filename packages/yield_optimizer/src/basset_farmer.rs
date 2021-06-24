use cosmwasm_std::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::Uint256;
use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub token_code_id: u64,
    pub casset_staking_code_id: u64,
    pub basset_token_addr: String,
    //Luna / ETH / Sol, will be converted to cLuna, cETH, cSol
    pub collateral_token_symbol: String,
    pub governance_addr: String,
    pub custody_basset_contract: String,
    pub anchor_token: String,
    pub anchor_market_contract: String,
    pub anchor_overseer_contract: String,
    pub anc_stable_swap_contract: String,
    pub psi_stable_swap_contract: String,
    pub aterra_token: String,
    pub psi_part_in_rewards: Decimal,
    pub psi_token: String,
    pub basset_farmer_config_contract: String,
    pub stable_denom: String,
    pub claiming_rewards_delay: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Anyone { anyone_msg: AnyoneMsg },
    Receive(Cw20ReceiveMsg),
    Yourself { yourself_msg: YourselfMsg },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum YourselfMsg {
    SwapAnc,
    DisributeRewards,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AnyoneMsg {
    HonestWork,
    Rebalance,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    Deposit,
    Withdraw,
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config,
    Rebalance,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub governance_contract: String,
    pub casset_staking_contract: String,
    pub anchor_token: String,
    pub anchor_overseer_contract: String,
    pub anchor_market_contract: String,
    pub custody_basset_contract: String,
    pub anc_stable_swap_contract: String,
    pub psi_stable_swap_contract: String,
    pub casset_token: String,
    pub basset_token: String,
    pub aterra_token: String,
    //what part of profit from selling ANC spend to buy PSI
    pub psi_part_in_rewards: Decimal,
    pub psi_token: String,
    pub basset_farmer_config_contract: String,
    pub stable_denom: String,
    pub claiming_rewards_delay: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum RebalanceResponse {
    Nothing,
    Borrow {
        amount: Uint256,
        advised_buffer_size: Uint256,
        is_possible: bool,
    },
    Repay {
        amount: Uint256,
        advised_buffer_size: Uint256,
    },
}
