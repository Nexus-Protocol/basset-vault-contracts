use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Addr, CanonicalAddr, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub governance_contract: Addr,
    pub overseer_contract: Addr,
    pub anchor_token: Addr,
    pub anchor_market_contract: Addr,
    pub custody_basset_contract: Addr,
    pub anchor_ust_swap_contract: Addr,
    pub ust_psi_swap_contract: Addr,
    pub casset_token: Addr,
    pub basset_token: Addr,
    pub aterra_token: Addr,
    //what part of UST from selling ANC spend to buy PSI
    pub psi_part_in_rewards: Uint128,
    pub psi_token: Addr,
    pub basset_farmer_config_contract: Addr,
    pub stable_denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    // pub last_reward_updated: u64,
    pub global_reward_index: Decimal256,
    pub last_reward_amount: Decimal256,
    //TODO: rename to avoid UST in naming
    pub ust_buffer_balance: Uint256,
    pub aterra_balance: Uint256,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
pub const FARMERS: Map<&Addr, FarmerInfo> = Map::new("farmers");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct FarmerInfo {
    //TODO: probably I don't need that field. We know balance from cAsset contract address
    //but I leave it here for some time
    pub balance_casset: Uint256,
    pub spendable_basset: Uint256,
}

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}

pub fn load_farmer_info(storage: &dyn Storage, farmer_addr: &Addr) -> StdResult<FarmerInfo> {
    FARMERS
        .may_load(storage, farmer_addr)
        .map(|res| res.unwrap_or_default())
}

pub fn store_farmer_info(
    storage: &mut dyn Storage,
    farmer_addr: &Addr,
    farmer_info: &FarmerInfo,
) -> StdResult<()> {
    FARMERS.save(storage, farmer_addr, farmer_info)
}

pub fn load_state(storage: &dyn Storage) -> StdResult<State> {
    STATE.load(storage)
}
