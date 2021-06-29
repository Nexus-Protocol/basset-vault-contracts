use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{Addr, StdResult, Storage};
use cw_storage_plus::Item;

pub const REWARDS_CONTRACT: Item<Addr> = Item::new("rewards_contract");

pub fn load_rewards_contract(storage: &dyn Storage) -> StdResult<Addr> {
    REWARDS_CONTRACT.load(storage)
}

pub fn save_rewards_contract(storage: &mut dyn Storage, addr: &Addr) -> StdResult<()> {
    REWARDS_CONTRACT.save(storage, addr)
}
