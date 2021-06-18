use cosmwasm_std::{Addr, Decimal};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub token_code_id: u64,
    //address for bLuna token, for example
    pub basset_token_addr: String,
    //Luna / ETH / Sol, will be converted to cLuna, cETH, cSol
    pub collateral_token_symbol: String,
    //Nexus overseer addr
    pub overseer_addr: String,
    pub governance_addr: String,
    pub custody_basset_contract: String,
    pub anchor_token: String,
    pub anchor_market_contract: String,
    pub anchor_ust_swap_contract: String,
    pub ust_psi_swap_contract: String,
    pub aterra_token: String,
    pub psi_part_in_rewards: Decimal,
    pub psi_token: String,
    pub basset_farmer_config_contract: String,
    pub stable_denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Anyone { anyone_msg: AnyoneMsg },
    OverseerMsg { overseer_msg: OverseerMsg },
    Receive(Cw20ReceiveMsg),
    Yourself { yourself_msg: YourselfMsg },
    // WithdrawTokens {
    //     depositor: Addr,
    //     amount: Uint256,
    // }
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
    ClaimRewards,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OverseerMsg {
    Deposit { farmer: String, amount: Uint256 },
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
    pub overseer_contract: String,
    pub custody_basset_contract: String,
    pub casset_token: String,
    pub basset_token: String,
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
