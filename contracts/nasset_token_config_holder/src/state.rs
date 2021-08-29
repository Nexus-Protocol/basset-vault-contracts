use basset_vault::nasset_token_config_holder::Config;
use cosmwasm_std::{Addr, StdResult, Storage};
use cw_storage_plus::Item;

static KEY_CONFIG: Item<Config> = Item::new("config");

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    KEY_CONFIG.load(storage)
}

pub fn save_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    KEY_CONFIG.save(storage, config)
}

pub fn set_nasset_token_rewards_contract(
    storage: &mut dyn Storage,
    addr: Addr,
) -> StdResult<Config> {
    KEY_CONFIG.update(storage, |mut cfg: Config| -> StdResult<_> {
        cfg.nasset_token_rewards_contract = addr;
        Ok(cfg)
    })
}
