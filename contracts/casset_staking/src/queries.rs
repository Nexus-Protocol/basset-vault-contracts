use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Deps, Env, StdResult};
use serde::__private::de::TagOrContentField;
use yield_optimizer::{
    basset_farmer_config::{query_borrower_action, BorrowerActionResponse},
    casset_staking::ConfigResponse,
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
        casset_token: config.casset_token.to_string(),
        aterra_token: config.aterra_token.to_string(),
        stable_denom: config.stable_denom,
        basset_farmer_contract: config.basset_farmer_contract.to_string(),
        anchor_market_contract: config.anchor_market_contract.to_string(),
    })
}
