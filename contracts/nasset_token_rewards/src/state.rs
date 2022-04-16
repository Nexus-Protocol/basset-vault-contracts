use cw_storage_plus::{Item, Map};
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GovernanceUpdateState {
    pub new_governance_contract_addr: Addr,
    pub wait_approve_until: u64,
}

static KEY_CONFIG: Item<Config> = Item::new("config");
static KEY_STATE: Item<State> = Item::new("state");
static KEY_GOVERNANCE_UPDATE: Item<GovernanceUpdateState> = Item::new("gov_update");
pub(crate) static HOLDERS: Map<&Addr, Holder> = Map::new("state");

pub fn load_state(storage: &dyn Storage) -> StdResult<State> {
    KEY_STATE.load(storage)
}

pub fn save_state(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
    KEY_STATE.save(storage, state)
}

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    KEY_CONFIG.load(storage)
}

pub fn save_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    KEY_CONFIG.save(storage, config)
}

pub fn load_holder(storage: &dyn Storage, addr: &Addr) -> StdResult<Holder> {
    HOLDERS
        .may_load(storage, addr)
        .map(|res| res.unwrap_or_default())
}

pub fn save_holder(storage: &mut dyn Storage, addr: &Addr, holder: &Holder) -> StdResult<()> {
    HOLDERS.save(storage, addr, holder)
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

pub fn remove_gov_update(storage: &mut dyn Storage) {
    KEY_GOVERNANCE_UPDATE.remove(storage)
}
