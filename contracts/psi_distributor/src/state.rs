use cw_storage_plus::Item;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{Addr, Binary, Deps, QueryRequest, StdResult, Storage, WasmQuery};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    pub psi_token: Addr,
    pub governance_contract: Addr,
    pub nasset_token_rewards_contract: Addr,
    pub community_pool_contract: Addr,
    pub basset_vault_strategy_contract: Addr,
    pub nasset_psi_swap_contract_addr: Addr,
    pub manual_ltv: Decimal256,
    pub fee_rate: Decimal256,
    pub tax_rate: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GovernanceUpdateState {
    pub new_governance_contract_addr: Addr,
    pub wait_approve_until: u64,
}

static KEY_CONFIG: Item<Config> = Item::new("config");
static KEY_GOVERNANCE_UPDATE: Item<GovernanceUpdateState> = Item::new("gov_update");

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    KEY_CONFIG.load(storage)
}

pub fn save_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    KEY_CONFIG.save(storage, config)
}

pub fn load_gov_update(storage: &dyn Storage) -> StdResult<GovernanceUpdateState> {
    KEY_GOVERNANCE_UPDATE.load(storage)
}

pub fn save_gov_update(
    storage: &mut dyn Storage,
    gov_update: &GovernanceUpdateState,
) -> StdResult<()> {
    KEY_GOVERNANCE_UPDATE.save(storage, gov_update)
}

pub fn remove_gov_update(storage: &mut dyn Storage) -> () {
    KEY_GOVERNANCE_UPDATE.remove(storage)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct BassetStrategyConfig {
    pub borrow_ltv_aim: Decimal256,
}

pub fn load_aim_ltv(deps: Deps, config: &Config) -> StdResult<Decimal256> {
    let basset_strategy_config: BassetStrategyConfig =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Raw {
            contract_addr: config.basset_vault_strategy_contract.to_string(),
            key: Binary::from(b"config"),
        }))?;

    Ok(basset_strategy_config.borrow_ltv_aim)
}
