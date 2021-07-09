use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub nasset_token_code_id: u64,
    pub nasset_token_config_holder_code_id: u64,
    pub nasset_token_rewards_code_id: u64,
    pub psi_distributor_code_id: u64,
    pub basset_token_addr: String,
    //Luna / ETH / Sol, will be converted to nLuna, nETH, nSol
    pub collateral_token_symbol: String,
    pub governance_contract: String,
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
    pub nasset_token_holders_psi_rewards_share: u64,
    pub governance_contract_psi_rewards_share: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Anyone { anyone_msg: AnyoneMsg },
    Receive(Cw20ReceiveMsg),
    Yourself { yourself_msg: YourselfMsg },
    Governance { governance_msg: GovernanceMsg },
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
    // Because basset_farmer always have more UST than loan,
    // then when last user will withdraw bAsset some UST remains in contract.
    // This command utilise it.
    ClaimRemainder,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceMsg {
    UpdateConfig {
        governance_contract_addr: Option<String>,
        psi_distributor_addr: Option<String>,
        anchor_token_addr: Option<String>,
        anchor_overseer_contract_addr: Option<String>,
        anchor_market_contract_addr: Option<String>,
        anchor_custody_basset_contract_addr: Option<String>,
        anc_stable_swap_contract_addr: Option<String>,
        psi_stable_swap_contract_addr: Option<String>,
        nasset_token_addr: Option<String>,
        basset_token_addr: Option<String>,
        aterra_token_addr: Option<String>,
        psi_token_addr: Option<String>,
        basset_farmer_config_contract_addr: Option<String>,
        stable_denom: Option<String>,
        claiming_rewards_delay: Option<u64>,
        over_loan_balance_value: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    Deposit,
    Withdraw,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config,
    Rebalance,
    ChildContractsCodeId,
    IsRewardsClaimable,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub governance_contract: String,
    pub anchor_token: String,
    pub anchor_overseer_contract: String,
    pub anchor_market_contract: String,
    pub custody_basset_contract: String,
    pub anc_stable_swap_contract: String,
    pub psi_stable_swap_contract: String,
    pub nasset_token: String,
    pub basset_token: String,
    pub aterra_token: String,
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ChildContractsInfoResponse {
    pub nasset_token_code_id: u64,
    pub nasset_token_rewards_code_id: u64,
    pub psi_distributor_code_id: u64,
    pub collateral_token_symbol: String,
    pub nasset_token_holders_psi_rewards_share: u64,
    pub governance_contract_psi_rewards_share: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IsRewardsClaimableResponse {
    pub claimable: bool,
    pub anc_amount: Decimal256,
    pub last_claiming_height: u64,
    pub current_height: u64,
}
