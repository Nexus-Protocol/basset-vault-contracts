use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Deps, StdResult};
use terra_cosmwasm::TerraQuerier;

pub struct TaxInfo {
    pub rate: Decimal256,
    pub cap: Uint256,
}

impl TaxInfo {
    #[cfg(not(test))]
    pub fn get_tax_for(&self, amount: Uint256) -> Uint256 {
        let tax_amount = if amount.is_zero() {
            Decimal256::zero()
        } else {
            Decimal256::from_uint256(amount)
                * (Decimal256::one() - Decimal256::one() / (Decimal256::one() + self.rate))
        };

        let tax_amount = TaxInfo::round_up(tax_amount);
        let tax_capped = std::cmp::min(tax_amount, self.cap);
        std::cmp::max(tax_capped, Uint256::one())
    }

    // #[cfg(not(test))]
    fn round_up(decimal: Decimal256) -> Uint256 {
        if decimal.is_zero() {
            Uint256::zero()
        } else {
            let truncated =
                (decimal.0 / Decimal256::DECIMAL_FRACTIONAL) * Decimal256::DECIMAL_FRACTIONAL;
            Uint256((truncated + Decimal256::DECIMAL_FRACTIONAL) / Decimal256::DECIMAL_FRACTIONAL)
        }
    }

    #[cfg(test)]
    pub fn get_tax_for(&self, amount: Uint256) -> Uint256 {
        if self.rate.is_zero() {
            return Uint256::zero();
        }

        let tax_amount = if amount.is_zero() {
            Decimal256::zero()
        } else {
            Decimal256::from_uint256(amount)
                * (Decimal256::one() - Decimal256::one() / (Decimal256::one() + self.rate))
        };
        println!("jjjjjjjj");
        let tax_amount = TaxInfo::round_up(tax_amount);
        println!("jjjjjjjj_2");
        let tax_capped = std::cmp::min(tax_amount, self.cap);
        println!("jjjjjjjj_3");

        std::cmp::max(tax_capped, Uint256::one())
    }

    // #[cfg(test)]
    // fn round_up(&self, decimal: Decimal256) -> Uint256 {
    //     if self.rate.is_zero() {
    //         return Uint256::zero();
    //     }

    //     if decimal.is_zero() {
    //         Uint256::zero()
    //     } else {
    //         let truncated =
    //             (decimal.0 / Decimal256::DECIMAL_FRACTIONAL) * Decimal256::DECIMAL_FRACTIONAL;
    //         Uint256((truncated + Decimal256::DECIMAL_FRACTIONAL) / Decimal256::DECIMAL_FRACTIONAL)
    //     }
    // }

    #[cfg(test)]
    pub fn get_revert_tax(&self, amount: Uint256) -> Uint256 {
        if self.rate.is_zero() {
            return Uint256::zero();
        }

        if amount.is_zero() {
            return Uint256::zero();
        }
        let tax_amount = amount * self.rate;
        let tax_capped = std::cmp::min(tax_amount, self.cap);
        std::cmp::max(tax_capped, Uint256::one())
    }

    #[cfg(not(test))]
    pub fn get_revert_tax(&self, amount: Uint256) -> Uint256 {
        if amount.is_zero() {
            return Uint256::zero();
        }
        let tax_amount = amount * self.rate;
        let tax_capped = std::cmp::min(tax_amount, self.cap);
        std::cmp::max(tax_capped, Uint256::one())
    }

    pub fn subtract_tax(&self, coin_amount: Uint256) -> Uint256 {
        println!(">>>>> coin_amount: {}", coin_amount);
        println!(
            ">>>>> self.get_tax_for(coin_amount): {}",
            self.get_tax_for(coin_amount)
        );
        if coin_amount.is_zero() {
            return Uint256::zero();
        }
        let res = coin_amount - self.get_tax_for(coin_amount);
        println!(">>>>> 2");
        res
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
        assert_eq!(tax, Uint256::from(319u64));
    }

    #[test]
    fn calcl_tax_to_send_million_coins_assert_from_terra_station() {
        let tax_info = TaxInfo {
            rate: Decimal256::from_str("0.003191811080725897").unwrap(),
            cap: Uint256::from(1_411_603u64),
        };
        let tax = tax_info.get_tax_for(Uint256::from(1_000_000u64));
        assert_eq!(tax, Uint256::from(3_182u64));
    }
}
