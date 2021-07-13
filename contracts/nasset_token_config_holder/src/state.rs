use cosmwasm_std::{Addr, StdResult, Storage};
use cosmwasm_storage::{singleton, singleton_read};
use yield_optimizer::nasset_token_config_holder::Config;

static KEY_CONFIG: &[u8] = b"config";

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, KEY_CONFIG).load()
}

pub fn save_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}

pub fn set_nasset_token_rewards_contract(
    storage: &mut dyn Storage,
    addr: Addr,
) -> StdResult<Config> {
    singleton(storage, KEY_CONFIG).update(|mut cfg: Config| -> StdResult<_> {
        cfg.nasset_token_rewards_contract = addr;
        Ok(cfg)
    })
}
