use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Deps, StdResult};
use terra_cosmwasm::TerraQuerier;

pub struct TaxInfo {
    pub rate: Decimal256,
    pub cap: Uint256,
}

impl TaxInfo {
    pub fn get_tax_for(&self, amount: Uint256) -> Uint256 {
        std::cmp::min(
            TaxInfo::round_up(Decimal256::from_uint256(amount) * self.rate),
            self.cap,
        )
    }

    fn round_up(decimal: Decimal256) -> Uint256 {
        let truncated =
            (decimal.0 / Decimal256::DECIMAL_FRACTIONAL) * Decimal256::DECIMAL_FRACTIONAL;
        Uint256((truncated + Decimal256::DECIMAL_FRACTIONAL) / Decimal256::DECIMAL_FRACTIONAL)
    }

    pub fn get_revert_tax(&self, amount: Uint256) -> Uint256 {
        std::cmp::min(amount * self.rate, self.cap)
    }

    pub fn subtract_tax(&self, coin_amount: Uint256) -> Uint256 {
        coin_amount - self.get_tax_for(coin_amount)
    }

    pub fn append_tax(&self, coin_amount: Uint256) -> Uint256 {
        coin_amount + self.get_revert_tax(coin_amount)
    }
}

pub fn get_tax_info(deps: Deps, coin_denom: &str) -> StdResult<TaxInfo> {
    let terra_querier = TerraQuerier::new(&deps.querier);
    let rate = Decimal256::from((terra_querier.query_tax_rate()?).rate);
    let cap = Uint256::from((terra_querier.query_tax_cap(coin_denom)?).cap);
    return Ok(TaxInfo { rate, cap });
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn calcl_tax_for_minimum_amount_to_send_assert_from_terra_station() {
        let tax_info = TaxInfo {
            rate: Decimal256::from_str("0.003191811080725897").unwrap(),
            cap: Uint256::from(1_411_603u64),
        };
        let tax = tax_info.get_tax_for(Uint256::one());
        assert_eq!(tax, Uint256::one());
    }

    #[test]
    fn calcl_tax_to_send_two_coins_assert_from_terra_station() {
        let tax_info = TaxInfo {
            rate: Decimal256::from_str("0.003191811080725897").unwrap(),
            cap: Uint256::from(1_411_603u64),
        };
        let tax = tax_info.get_tax_for(Uint256::from(2u64));
        assert_eq!(tax, Uint256::one());
    }

    #[test]
    fn calcl_tax_to_send_hundred_coins_assert_from_terra_station() {
        let tax_info = TaxInfo {
            rate: Decimal256::from_str("0.003191811080725897").unwrap(),
            cap: Uint256::from(1_411_603u64),
        };
        let tax = tax_info.get_tax_for(Uint256::from(100u64));
        assert_eq!(tax, Uint256::one());
    }

    #[test]
    fn calcl_tax_to_send_thousand_coins_assert_from_terra_station() {
        let tax_info = TaxInfo {
            rate: Decimal256::from_str("0.003191811080725897").unwrap(),
            cap: Uint256::from(1_411_603u64),
        };
        let tax = tax_info.get_tax_for(Uint256::from(1_000u64));
        assert_eq!(tax, Uint256::from(4u64));
    }

    #[test]
    fn calcl_tax_to_send_ten_thousands_coins_assert_from_terra_station() {
        let tax_info = TaxInfo {
            rate: Decimal256::from_str("0.003191811080725897").unwrap(),
            cap: Uint256::from(1_411_603u64),
        };
        let tax = tax_info.get_tax_for(Uint256::from(10_000u64));
        assert_eq!(tax, Uint256::from(32u64));
    }

    #[test]
    fn calcl_tax_to_send_hundred_thousands_coins_assert_from_terra_station() {
        let tax_info = TaxInfo {
            rate: Decimal256::from_str("0.003191811080725897").unwrap(),
            cap: Uint256::from(1_411_603u64),
        };
        let tax = tax_info.get_tax_for(Uint256::from(100_000u64));
        assert_eq!(tax, Uint256::from(320u64));
    }

    #[test]
    fn calcl_tax_to_send_million_coins_assert_from_terra_station() {
        let tax_info = TaxInfo {
            rate: Decimal256::from_str("0.003191811080725897").unwrap(),
            cap: Uint256::from(1_411_603u64),
        };
        let tax = tax_info.get_tax_for(Uint256::from(1_000_000u64));
        assert_eq!(tax, Uint256::from(3_192u64));
    }
}
