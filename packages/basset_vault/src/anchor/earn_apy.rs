use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{Deps, Addr, StdResult, QuerierWrapper, QueryRequest, WasmQuery, to_binary};
use serde::{Serialize, Deserialize};
use super::NUMBER_OF_BLOCKS_PER_YEAR;


#[derive(Deserialize)]
struct EpochState {
    deposit_rate: Decimal256,

    // We don't use these fields
    //
    // prev_aterra_supply: Uint256,
    // prev_exchange_rate: Decimal256,
    // prev_interest_buffer: Uint256,
    // last_executed_height: u64,
}

#[derive(Serialize)]
enum AnchorOverseerQueryMsg {
    EpochState {},

    // We don't use other cases
}

fn calculate_anchor_earn_apy(deposit_rate: Decimal256) -> Decimal256 {
    deposit_rate * Decimal256::from_uint256(NUMBER_OF_BLOCKS_PER_YEAR)
}

fn query_epoch_state(
    querier: &QuerierWrapper,
    anchor_overseer_contract: &Addr,
) -> StdResult<EpochState> {
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: anchor_overseer_contract.to_string(),
        msg: to_binary(&AnchorOverseerQueryMsg::EpochState {})?
    }))
}

pub fn query_anchor_earn_apr(
    deps: Deps,
    anchor_overseer_contract: &Addr,
) -> StdResult<Decimal256> {
    let epoch_state = query_epoch_state(&deps.querier, anchor_overseer_contract)?;
    let apy = calculate_anchor_earn_apy(epoch_state.deposit_rate);
    Ok(apy)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_calculate_anchor_earn_apy() {
        let deposit_rate = Decimal256::from_str("0.000000041734138975").unwrap();
        let apy = calculate_anchor_earn_apy(deposit_rate);
        assert_eq!(apy, Decimal256::from_str("0.19434795572016975").unwrap());
    }
}