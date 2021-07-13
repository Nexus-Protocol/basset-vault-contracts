use cosmwasm_std::{Addr, StdResult, Storage};
use cosmwasm_storage::{singleton, singleton_read};

static KEY_CONFIG_HOLDER_CONTRACT: &[u8] = b"config_holder_contract";

pub fn load_config_holder_contract(storage: &dyn Storage) -> StdResult<Addr> {
    singleton_read(storage, KEY_CONFIG_HOLDER_CONTRACT).load()
}

pub fn save_config_holder_contract(storage: &mut dyn Storage, addr: &Addr) -> StdResult<()> {
    singleton(storage, KEY_CONFIG_HOLDER_CONTRACT).save(addr)
}
