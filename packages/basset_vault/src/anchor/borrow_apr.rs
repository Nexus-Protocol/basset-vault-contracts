use super::market::query_market_state;
use crate::querier::query_balance;
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{StdResult, Deps, Addr, QueryRequest, WasmQuery, to_binary, QuerierWrapper, StdError};
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use super::NUMBER_OF_BLOCKS_PER_YEAR;
use crate::astroport_pair::{QueryMsg as AstroportPairQueryMsg, PoolResponse};

fn calculate_anchor_borrow_distribution_apr(
    anc_price: Decimal256,
    anc_emission_rate: Decimal256,
    total_liabilities: Decimal256,
) -> StdResult<Decimal256> {
    if total_liabilities.is_zero() {
        return Err(StdError::generic_err("Total liabilities is zero"));
    }
    Ok(anc_emission_rate * anc_price * NUMBER_OF_BLOCKS_PER_YEAR / total_liabilities)
}

fn calculate_anchor_borrow_interest_apr(borrow_rate: Decimal256) -> Decimal256 {
    borrow_rate * NUMBER_OF_BLOCKS_PER_YEAR
}

#[derive(Debug, Clone, Copy)]
pub struct BorrowNetApr {
    pub distribution_apr: Decimal256,
    pub interest_apr: Decimal256,
}

fn calculate_anchor_borrow_net_apr(
    anc_price: Decimal256,
    anc_emission_rate: Decimal256,
    total_liabilities: Decimal256,
    borrow_rate: Decimal256,
) -> StdResult<BorrowNetApr> {
    let distribution_apr = calculate_anchor_borrow_distribution_apr(
        anc_price,
        anc_emission_rate,
        total_liabilities,
    )?;

    let interest_apr = calculate_anchor_borrow_interest_apr(borrow_rate);

    Ok(BorrowNetApr {
        distribution_apr,
        interest_apr,
    })
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum InterestModelQueryMsg {
    BorrowRate {
        market_balance: Uint256,
        total_liabilities: Decimal256,
        total_reserves: Decimal256,
    },

    // Unused
    // Config {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BorrowRateResponse {
    pub rate: Decimal256,
}

fn query_borrow_rate(
    querier: &QuerierWrapper,
    anchor_market_contract: &Addr,
    stable_denom: String,
    anchor_interest_model_contract: &Addr,
    total_liabilities: Decimal256,
    total_reserves: Decimal256,
) -> StdResult<BorrowRateResponse> {
    let market_balance = query_balance(&querier, anchor_market_contract, stable_denom)?;
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: anchor_interest_model_contract.to_string(),
        msg: to_binary(&InterestModelQueryMsg::BorrowRate {
            market_balance: market_balance.into(),
            total_liabilities,
            total_reserves,
        })?
    }))
}

fn query_anc_price(
    querier: &QuerierWrapper,
    anc_ust_swap_contract: &Addr,
) -> StdResult<Decimal256> {
    let pool: PoolResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: anc_ust_swap_contract.to_string(),
        msg: to_binary(&AstroportPairQueryMsg::Pool {})?
    }))?;

    let [anc_asset, ust_asset] = pool.assets;

    if anc_asset.amount.is_zero() {
        return Err(StdError::generic_err("Can't calculate ANC price, ANC amount in pool = 0"));
    }

    Ok(Decimal256::from_ratio(Uint256::from(ust_asset.amount), Uint256::from(anc_asset.amount)))
}

pub fn query_anchor_borrow_net_apr(
    deps: Deps,
    anchor_market_contract: &Addr,
    anchor_interest_model_contract: &Addr,
    anc_ust_swap_contract: &Addr,
    stable_denom: String,
) -> StdResult<BorrowNetApr> {
    let market_state = query_market_state(deps, anchor_market_contract)?;

    let borrow_rate = query_borrow_rate(
        &deps.querier,
        anchor_market_contract,
        stable_denom,
        anchor_interest_model_contract,
        market_state.total_liabilities,
        market_state.total_reserves,
    )?;

    let anc_price = query_anc_price(&deps.querier, anc_ust_swap_contract)?;

    let net_apr = calculate_anchor_borrow_net_apr(
        anc_price,
        market_state.anc_emission_rate,
        market_state.total_liabilities,
        borrow_rate.rate,
    )?;

    Ok(net_apr)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_calculate_anchor_borrow_distribution_apr() {
        let anc_price = Decimal256::from_str("2.909").unwrap();
        let anc_emission_rate = Decimal256::from_str("20381363.85157231012364762").unwrap();
        let total_liabilities = Decimal256::from_str("682703198917970.293507449953151794").unwrap();
        let apr = calculate_anchor_borrow_distribution_apr(anc_price, anc_emission_rate, total_liabilities).unwrap();
        assert_eq!(apr, Decimal256::from_str("0.40442085635709844").unwrap());
    }

    #[test]
    fn test_calculate_anchor_borrow_distribution_apr2() {
        let anc_price = Decimal256::from_str("3.95").unwrap();
        let anc_emission_rate = Decimal256::from_str("20381363.85157231012364762").unwrap();
        let total_liabilities = Decimal256::from_str("1648733099209250.164427424496482224").unwrap();
        let apr = calculate_anchor_borrow_distribution_apr(anc_price, anc_emission_rate, total_liabilities).unwrap();
        assert_eq!(apr, Decimal256::from_str("0.227388501644376033").unwrap());
    }

    #[test]
    fn test_calculate_anchor_borrow_distribution_apr3() {
        let anc_price = Decimal256::from_str("3.95").unwrap();
        let anc_emission_rate = Decimal256::from_str("20381363.85157231012364762").unwrap();
        let total_liabilities = Decimal256::from_str("0.0").unwrap();
        let apr = calculate_anchor_borrow_distribution_apr(anc_price, anc_emission_rate, total_liabilities);
        assert_eq!(apr.unwrap_err().to_string(), "Generic error: Total liabilities is zero");
    }

    #[test]
    fn test_calculate_anchor_borrow_interest_apr() {
        let borrow_rate = Decimal256::from_str("0.000000047824728815").unwrap();
        let apr = calculate_anchor_borrow_interest_apr(borrow_rate);
        assert_eq!(apr, Decimal256::from_str("0.22271067539298015").unwrap());
    }

    #[test]
    fn test_calculate_anchor_borrow_net_apr() {
        let anc_price = Decimal256::from_str("2.909").unwrap();
        let anc_emission_rate = Decimal256::from_str("20381363.85157231012364762").unwrap();
        let total_liabilities = Decimal256::from_str("682703198917970.293507449953151794").unwrap();
        let borrow_rate = Decimal256::from_str("0.000000047824728815").unwrap();
        let apr = calculate_anchor_borrow_net_apr(anc_price, anc_emission_rate, total_liabilities, borrow_rate).unwrap();
        assert_eq!(apr.distribution_apr - apr.interest_apr, Decimal256::from_str("0.18171018096411829").unwrap());
    }

    #[test]
    fn test_calculate_anchor_borrow_net_apr2() {
        let anc_price = Decimal256::from_str("2.909").unwrap();
        let anc_emission_rate = Decimal256::from_str("20381363.85157231012364762").unwrap();
        let total_liabilities = Decimal256::from_str("682703198917970.293507449953151794").unwrap();
        let borrow_rate = Decimal256::from_str("1.0").unwrap(); // high borrow rate to make borrow apr negative
        let apr = calculate_anchor_borrow_net_apr(anc_price, anc_emission_rate, total_liabilities, borrow_rate).unwrap();
        assert!(apr.distribution_apr < apr.interest_apr);
    }
}
