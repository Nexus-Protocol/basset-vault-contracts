use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::Decimal256;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub psi_token_addr: String,
    pub governance_contract_addr: String,
    pub nasset_token_rewards_contract_addr: String,
    pub community_pool_contract_addr: String,
    pub basset_vault_strategy_contract_addr: String,
    pub manual_ltv: Decimal256,
    pub fee_rate: Decimal256,
    pub tax_rate: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Anyone { anyone_msg: AnyoneMsg },
    GovernanceMsg { governance_msg: GovernanceMsg },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AnyoneMsg {
    DistributeRewards {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceMsg {
    UpdateConfig {
        governance_contract_addr: Option<String>,
        nasset_token_rewards_contract_addr: Option<String>,
        community_pool_contract_addr: Option<String>,
        basset_vault_strategy_contract_addr: Option<String>,
        manual_ltv: Option<Decimal256>,
        fee_rate: Option<Decimal256>,
        tax_rate: Option<Decimal256>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub psi_token_addr: String,
    pub governance_contract_addr: String,
    pub nasset_token_rewards_contract_addr: String,
    pub community_pool_contract_addr: String,
    pub basset_vault_strategy_contract_addr: String,
    pub manual_ltv: Decimal256,
    pub fee_rate: Decimal256,
    pub tax_rate: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
