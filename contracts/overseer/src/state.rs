use cosmwasm_storage::{singleton, singleton_read};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Addr, Api, CanonicalAddr, Order, StdError, StdResult, Storage};
use cw_storage_plus::Bound;
use cw_storage_plus::Item;
use cw_storage_plus::Map;
use yield_optimizer::asset::{AssetInfo, AssetInfoRaw};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub maintainer: CanonicalAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WhitelistElem {
    pub name: String,
    pub symbol: String,
    pub max_ltv: Decimal256,
    pub vault_contract: CanonicalAddr,
}

pub const STATE: Item<State> = Item::new("state");
pub const DEPOSITED_ASSETS: Map<Addr, Vec<AssetInfoRaw>> = Map::new("deposits");
pub const TOKENS_WHITELIST: Map<Addr, WhitelistElem> = Map::new("whitelist");
pub const CONFIG: Item<Config> = Item::new("config");

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    CONFIG.save(storage, config)
}
pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}

pub fn store_deposits(
    storage: &mut dyn Storage,
    depositor: &CanonicalAddr,
    assets: &Vec<AssetInfoRaw>,
) -> StdResult<()> {
    // if assets.len() == 0 {
    //     DEPOSITED_ASSETS.remove(storage, depositor.as_slice());
    // } else {
    //     DEPOSITED_ASSETS.save(storage, depositor.as_slice(), assets)?;
    // }

    Ok(())
}

pub fn read_deposits(
    storage: &dyn Storage,
    depositor: &CanonicalAddr,
) -> StdResult<Vec<AssetInfoRaw>> {
    Ok(vec![])
    // match DEPOSITED_ASSETS.may_load(storage, depositor.as_slice())? {
    //     Some(v) => Ok(v),
    //     None => Ok(vec![]),
    // }
}

// pub fn depositor_to_human(
//     api: &dyn Api,
//     depositor: StdResult<(Vec<u8>, TokensCanonical)>,
// ) -> StdResult<(Addr, TokensAddr)> {
//     let (canonical_addr_bytes, tokens) = depositor?;
//     let address_human = api.addr_humanize(&CanonicalAddr::from(canonical_addr_bytes))?;
//     let tokens_human = tokens.to_human(api)?;
//     Ok((address_human, tokens_human))
// }

pub fn read_whitelist_elem(
    storage: &dyn Storage,
    token: &CanonicalAddr,
) -> StdResult<WhitelistElem> {
    Err(StdError::generic_err("Token is not registered"))
    // match TOKENS_WHITELIST.load(storage, token.as_slice()) {
    //     Ok(v) => Ok(v),
    //     _ => Err(StdError::generic_err("Token is not registered")),
    // }
}
