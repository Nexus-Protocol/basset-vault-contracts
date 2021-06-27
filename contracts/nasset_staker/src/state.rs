use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Addr, StdResult, Storage};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub nasset_token: Addr,
    pub psi_token: Addr,
    pub governance_addr: Addr,
}

const CONFIG: Item<Config> = Item::new("config");
const STATE: Item<State> = Item::new("state");
const STAKERS: Map<&Addr, StakerState> = Map::new("stakers");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct State {
    pub global_reward_index: Decimal256,
    pub last_reward_amount: Uint256,
    pub total_staked_amount: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct StakerState {
    pub staked_amount: Uint256,
    pub reward_index: Decimal256,
    pub pending_rewards: Decimal256,
}

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    CONFIG.save(storage, config)
}

pub fn config_set_nasset_token(storage: &mut dyn Storage, nasset_token: Addr) -> StdResult<Config> {
    CONFIG.update(storage, |mut config| -> StdResult<_> {
        config.nasset_token = nasset_token;
        Ok(config)
    })
}

pub fn load_staker_state(storage: &dyn Storage, addr: &Addr) -> StdResult<StakerState> {
    STAKERS
        .may_load(storage, addr)
        .map(|res| res.unwrap_or_default())
}

pub fn store_staker_state(
    storage: &mut dyn Storage,
    addr: &Addr,
    state: &StakerState,
) -> StdResult<()> {
    STAKERS.save(storage, addr, state)
}

pub fn load_state(storage: &dyn Storage) -> StdResult<State> {
    STATE.load(storage)
}

pub fn store_state(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
    STATE.save(storage, state)
}
