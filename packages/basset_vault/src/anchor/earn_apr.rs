use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{Deps, Addr, StdResult, QuerierWrapper, QueryRequest, WasmQuery, Binary};
use cosmwasm_storage::to_length_prefixed;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use super::NUMBER_OF_BLOCKS_PER_YEAR;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
struct EpochState {
    deposit_rate: Decimal256,

    // We don't use these fields
    //
    // prev_aterra_supply: Uint256,
    // prev_exchange_rate: Decimal256,
    // prev_interest_buffer: Uint256,
    // last_executed_height: u64,
}

fn query_epoch_state(
    querier: &QuerierWrapper,
    anchor_overseer_contract: &Addr,
) -> StdResult<EpochState> {
    querier.query(&QueryRequest::Wasm(WasmQuery::Raw {
        contract_addr: anchor_overseer_contract.to_string(),
        // Anchor use cosmwasm_storage::Singleton which add length prefix
        key: Binary::from(to_length_prefixed(b"epoch_state").to_vec()),
    }))
}

fn calculate_anchor_earn_apr(deposit_rate: Decimal256) -> Decimal256 {
    deposit_rate * NUMBER_OF_BLOCKS_PER_YEAR
}

pub fn query_anchor_earn_apr(
    deps: Deps,
    anchor_overseer_contract: &Addr,
) -> StdResult<Decimal256> {
    let epoch_state = query_epoch_state(&deps.querier, anchor_overseer_contract)?;
    let apr = calculate_anchor_earn_apr(epoch_state.deposit_rate);
    Ok(apr)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_calculate_anchor_earn_apr() {
        let deposit_rate = Decimal256::from_str("0.000000041734138975").unwrap();
        let apr = calculate_anchor_earn_apr(deposit_rate);
        assert_eq!(apr, Decimal256::from_str("0.19434795572016975").unwrap());
    }
}
