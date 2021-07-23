use cosmwasm_storage::{singleton, singleton_read, to_length_prefixed};
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{Addr, Binary, Deps, QueryRequest, StdResult, Storage, WasmQuery};
use schemars::JsonSchema;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    pub psi_token: Addr,
    pub governance_contract: Addr,
    pub nasset_token_rewards_contract: Addr,
    pub community_pool_contract: Addr,
    pub basset_vault_strategy_contract: Addr,
    pub manual_ltv: Decimal256,
    pub fee_rate: Decimal256,
    pub tax_rate: Decimal256,
}

static KEY_CONFIG: &[u8] = b"config";

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, KEY_CONFIG).load()
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub(crate) struct BassetStrategyConfig {
    pub borrow_ltv_aim: Decimal256,
}

pub fn load_aim_ltv(deps: Deps, config: &Config) -> StdResult<Decimal256> {
    let basset_strategy_config: BassetStrategyConfig =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Raw {
            contract_addr: config.basset_vault_strategy_contract.to_string(),
            key: Binary::from(to_length_prefixed(b"config").to_vec()),
        }))?;

    Ok(basset_strategy_config.borrow_ltv_aim)
}
