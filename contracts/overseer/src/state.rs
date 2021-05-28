use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Addr, Api, CanonicalAddr, Order, StdError, StdResult, Storage};
use cw_storage_plus::Bound;
use cw_storage_plus::Item;
use cw_storage_plus::Map;
use yield_optimizer::tokens::{Tokens, TokensHuman, TokensToHuman};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WhitelistElem {
    pub name: String,
    pub symbol: String,
    pub max_ltv: Decimal256,
    pub custody_contract: CanonicalAddr,
}

pub const STATE: Item<State> = Item::new("state");
//key: CanonicalAddr
pub const DEPOSITED_TOKENS: Map<&[u8], Tokens> = Map::new("deposits");
//key: CanonicalAddr
pub const TOKENS_WHITELIST: Map<&[u8], WhitelistElem> = Map::new("whitelist");

pub fn store_deposits(
    storage: &mut dyn Storage,
    depositor: &CanonicalAddr,
    tokens: &Tokens,
) -> StdResult<()> {
    if tokens.len() == 0 {
        DEPOSITED_TOKENS.remove(storage, depositor.as_slice());
    } else {
        DEPOSITED_TOKENS.save(storage, depositor.as_slice(), tokens)?;
    }

    Ok(())
}

pub fn read_deposits(storage: &dyn Storage, depositor: &CanonicalAddr) -> StdResult<Tokens> {
    match DEPOSITED_TOKENS.may_load(storage, depositor.as_slice())? {
        Some(v) => Ok(v),
        None => Ok(vec![]),
    }
}

pub fn depositor_to_human(
    api: &dyn Api,
    depositor: StdResult<(Vec<u8>, Tokens)>,
) -> StdResult<(Addr, TokensHuman)> {
    let (canonical_addr_bytes, tokens) = depositor?;
    let address_human = api.addr_humanize(&CanonicalAddr::from(canonical_addr_bytes))?;
    let tokens_human = tokens.to_human(api)?;
    Ok((address_human, tokens_human))
}

pub fn read_whitelist_elem(
    storage: &dyn Storage,
    token: &CanonicalAddr,
) -> StdResult<WhitelistElem> {
    match TOKENS_WHITELIST.load(storage, token.as_slice()) {
        Ok(v) => Ok(v),
        _ => Err(StdError::generic_err("Token is not registered")),
    }
}
