use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Addr, CanonicalAddr, Decimal, StdResult, Storage};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub borrow_ration_aim: Decimal,
    pub borrow_ration_upper_gap: Decimal,
    pub borrow_ration_bottom_gap: Decimal,
    pub oracle_addr: Addr,
    pub basset_token_addr: Addr,
    pub stable_denom: String,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub prices: Vec<Decimal256>,
    pub price_last_update_time: u64,

    // 1. average price value
    // 2. std_dev from prices
    // 3. std_dev / avg_price * 100
    pub last_std_dev_from_average_price: Decimal256,
}

pub const STATE: Item<State> = Item::new("state");

pub fn load_state(storage: &dyn Storage) -> StdResult<State> {
    STATE.load(storage)
}

pub fn save_state(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
    STATE.save(storage, state)
}
