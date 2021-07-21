use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub governance_contract_addr: String,
    pub basset_token_addr: String,
    pub anchor_token_addr: String,
    pub anchor_market_contract_addr: String,
    pub anchor_overseer_contract_addr: String,
    pub anchor_custody_basset_contract_addr: String,
    pub anc_stable_swap_contract_addr: String,
    pub psi_stable_swap_contract_addr: String,
    pub aterra_token_addr: String,
    pub psi_token_addr: String,
    pub basset_vault_strategy_contract_addr: String,
    pub stable_denom: String,
    pub claiming_rewards_delay: u64,
    pub over_loan_balance_value: String,
    pub nasset_token_holders_psi_rewards_share: u64,
    pub governance_contract_psi_rewards_share: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Governance { governance_msg: GovernanceMsg },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceMsg {
    UpdateConfig {
        governance_contract_addr: Option<String>,
        anchor_overseer_contract_addr: Option<String>,
        anchor_market_contract_addr: Option<String>,
        anchor_custody_basset_contract_addr: Option<String>,
        anc_stable_swap_contract_addr: Option<String>,
        psi_stable_swap_contract_addr: Option<String>,
        basset_vault_strategy_contract_addr: Option<String>,
        claiming_rewards_delay: Option<u64>,
        over_loan_balance_value: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub governance_contract: Addr,
    pub anchor_token: Addr,
    pub anchor_overseer_contract: Addr,
    pub anchor_market_contract: Addr,
    pub anchor_custody_basset_contract: Addr,
    pub anc_stable_swap_contract: Addr,
    pub psi_stable_swap_contract: Addr,
    pub basset_token: Addr,
    pub aterra_token: Addr,
    pub psi_token: Addr,
    pub basset_vault_strategy_contract: Addr,
    pub stable_denom: String,
    pub claiming_rewards_delay: u64,
    //UST value in balance should be more than loan
    //on what portion.
    //for example: 1.01 means 1% more than loan
    pub over_loan_balance_value: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub governance_contract_addr: String,
    pub anchor_token_addr: String,
    pub anchor_overseer_contract_addr: String,
    pub anchor_market_contract_addr: String,
    pub anchor_custody_basset_contract_addr: String,
    pub anc_stable_swap_contract_addr: String,
    pub psi_stable_swap_contract_addr: String,
    pub basset_token_addr: String,
    pub aterra_token_addr: String,
    pub psi_token_addr: String,
    pub basset_vault_strategy_contract_addr: String,
    pub stable_denom: String,
    pub claiming_rewards_delay: u64,
    //UST value in balance should be more than loan
    //on what portion.
    //for example: 1.01 means 1% more than loan
    pub over_loan_balance_value: Decimal256,
}
