use cosmwasm_std::{Deps, StdResult};

use crate::state::{State, STATE};

pub fn query_state(deps: Deps) -> StdResult<State> {
    STATE.load(deps.storage)
}
