use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Addr, CanonicalAddr, Decimal, StdResult, Storage};
use cw_storage_plus::{Item, Map};
use std::collections::VecDeque;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub governance_contract_addr: Addr,
    pub borrow_ltv_max: Decimal256,
    pub borrow_ltv_min: Decimal256,
    pub borrow_ltv_aim: Decimal256,
    pub oracle_addr: Addr,
    pub basset_token_addr: Addr,
    pub stable_denom: String,
    pub basset_max_ltv: Decimal256,
    //TODO: looks like I don't need that
    pub price_timeframe_millis: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}

pub const PRICES_COUNT: u64 = 5;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    //TODO: simple f32 is enough here
    pub prices: VecDeque<Decimal256>,
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
