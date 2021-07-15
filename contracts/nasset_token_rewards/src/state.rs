use cosmwasm_storage::{singleton, singleton_read, Bucket, ReadonlyBucket};
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, StdResult, Storage, Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    pub psi_token: Addr,
    pub nasset_token: Addr,
    pub governance_contract: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct State {
    pub global_index: Decimal,
    pub total_balance: Uint128,
    pub prev_reward_balance: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Holder {
    pub balance: Uint128,
    pub index: Decimal,
    pub pending_rewards: Decimal,
}

static KEY_CONFIG: &[u8] = b"config";
static KEY_STATE: &[u8] = b"state";

pub(crate) static PREFIX_HOLDERS: &[u8] = b"holders";

pub fn load_state(storage: &dyn Storage) -> StdResult<State> {
    singleton_read(storage, KEY_STATE).load()
}

pub fn save_state(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
    singleton(storage, KEY_STATE).save(state)
}

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, KEY_CONFIG).load()
}

pub fn save_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}

pub fn load_holder(storage: &dyn Storage, addr: &Addr) -> StdResult<Holder> {
    ReadonlyBucket::<Holder>::new(storage, PREFIX_HOLDERS)
        .may_load(addr.as_bytes())
        .map(|res| res.unwrap_or_default())
}

pub fn save_holder(storage: &mut dyn Storage, addr: &Addr, holder: &Holder) -> StdResult<()> {
    Bucket::new(storage, PREFIX_HOLDERS).save(addr.as_bytes(), holder)
}
