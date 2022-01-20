use super::market::query_market_state;
use crate::querier::query_balance;
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{StdResult, Deps, Addr, QueryRequest, WasmQuery, to_binary, QuerierWrapper};
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use super::NUMBER_OF_BLOCKS_PER_YEAR;

fn calculate_anchor_borrow_distribution_apr(
    anc_price: Decimal256,
    anc_emission_rate: Decimal256,
    total_liabilities: Decimal256,
) -> Decimal256 {
    let blocks_per_year = Decimal256::from_uint256(NUMBER_OF_BLOCKS_PER_YEAR);
    let apr = anc_emission_rate * anc_price * blocks_per_year / total_liabilities;
    apr
}

fn calculate_anchor_borrow_interest_apr(borrow_rate: Decimal256) -> Decimal256 {
    let blocks_per_year = Decimal256::from_uint256(NUMBER_OF_BLOCKS_PER_YEAR);
    let apr = borrow_rate * blocks_per_year;
    apr
}

fn calculate_anchor_borrow_net_apr(
    anc_price: Decimal256,
    anc_emission_rate: Decimal256,
    total_liabilities: Decimal256,
    borrow_rate: Decimal256,
) -> Decimal256 {
    let distribution_apr = calculate_anchor_borrow_distribution_apr(
        anc_price,
        anc_emission_rate,
        total_liabilities,
    );

    let interest_apr = calculate_anchor_borrow_interest_apr(borrow_rate);

    let net_arp = distribution_apr - interest_apr;

    net_arp
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

pub fn query_anchor_borrow_net_apr(
    deps: Deps,
    anchor_market_contract: &Addr,
    anchor_interest_model_contract: &Addr,
    anc_price: Decimal256,
    stable_denom: String,
) -> StdResult<Decimal256> {
    let market_state = query_market_state(deps, anchor_market_contract)?;

    let borrow_rate = query_borrow_rate(
        &deps.querier,
        anchor_market_contract,
        stable_denom,
        anchor_interest_model_contract,
        market_state.total_liabilities,
        market_state.total_reserves,
    )?;

    let net_apr = calculate_anchor_borrow_net_apr(
        anc_price,
        market_state.anc_emission_rate,
        market_state.total_liabilities,
        borrow_rate.rate,
    );

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
        let apr = calculate_anchor_borrow_distribution_apr(anc_price, anc_emission_rate, total_liabilities);
        assert_eq!(apr, Decimal256::from_str("0.40442085635709844").unwrap());
    }

    #[test]
    fn test_calculate_anchor_borrow_distribution_apr2() {
        let anc_price = Decimal256::from_str("3.95").unwrap();
        let anc_emission_rate = Decimal256::from_str("20381363.85157231012364762").unwrap();
        let total_liabilities = Decimal256::from_str("1648733099209250.164427424496482224").unwrap();
        let apr = calculate_anchor_borrow_distribution_apr(anc_price, anc_emission_rate, total_liabilities);
        assert_eq!(apr, Decimal256::from_str("0.227388501644376033").unwrap());
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
        let apr = calculate_anchor_borrow_net_apr(anc_price, anc_emission_rate, total_liabilities, borrow_rate);
        assert_eq!(apr, Decimal256::from_str("0.18171018096411829").unwrap());
    }
}
