use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub psi_token: Addr,
    pub nasset_token: Addr,
    pub governance_contract: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub global_index: Decimal,
    pub total_balance: Uint128,
    pub prev_reward_balance: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct Holder {
    pub balance: Uint128,
    pub index: Decimal,
    pub pending_rewards: Decimal,
}

pub const STATE: Item<State> = Item::new("state");
pub const CONFIG: Item<Config> = Item::new("config");
pub const HOLDERS: Map<&Addr, Holder> = Map::new("holders");

pub fn load_state(storage: &dyn Storage) -> StdResult<State> {
    STATE.load(storage)
}

pub fn save_state(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
    STATE.save(storage, state)
}

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}

pub fn save_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    CONFIG.save(storage, config)
}

pub fn load_holder(storage: &dyn Storage, addr: &Addr) -> StdResult<Holder> {
    HOLDERS
        .may_load(storage, addr)
        .map(|res| res.unwrap_or_default())
}

pub fn save_holder(storage: &mut dyn Storage, addr: &Addr, holder: &Holder) -> StdResult<()> {
    HOLDERS.save(storage, addr, holder)
}
