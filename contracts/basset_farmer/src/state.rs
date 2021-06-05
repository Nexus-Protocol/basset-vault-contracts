use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{Addr, CanonicalAddr, StdResult, Storage};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub overseer_contract: Addr,
    pub custody_basset_contract: Addr,
    pub casset_token: Addr,
    pub basset_token: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");
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
