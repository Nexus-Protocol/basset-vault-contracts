use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Deps, Env, StdResult};
use yield_optimizer::{
    basset_farmer::{ConfigResponse, RebalanceResponse},
    basset_farmer_config::{query_borrower_action, BorrowerActionResponse},
    querier::{
        get_basset_in_custody, query_balance, query_borrower_info, query_market_config,
        query_market_state, AnchorMarketConfigResponse, AnchorMarketStateResponse,
        BorrowerInfoResponse,
    },
};

use crate::state::load_config;
use crate::state::Config;

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = load_config(deps.storage)?;
    Ok(ConfigResponse {
        governance_contract: config.governance_contract.to_string(),
        overseer_contract: config.overseer_contract.to_string(),
        custody_basset_contract: config.custody_basset_contract.to_string(),
        casset_token: config.casset_token.to_string(),
        basset_token: config.basset_token.to_string(),
    })
}
