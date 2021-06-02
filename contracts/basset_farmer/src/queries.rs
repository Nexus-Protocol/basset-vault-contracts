use cosmwasm_std::{Deps, StdResult};

use crate::state::load_config;
use crate::state::Config;

pub fn query_config(deps: Deps) -> StdResult<Config> {
    load_config(deps.storage)
}
