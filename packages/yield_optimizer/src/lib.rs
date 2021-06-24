use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Coin, Deps, StdResult};
use terra_cosmwasm::TerraQuerier;

pub mod basset_farmer;
pub mod basset_farmer_config;
pub mod casset_staking;
pub mod psi_distributor;
pub mod querier;
pub mod terraswap;
pub mod terraswap_pair;

pub fn deduct_tax(deps: Deps, coin: Coin) -> StdResult<Coin> {
    let tax_info = get_tax_info(deps, &coin.denom)?;
    let coin_amount = Uint256::from(coin.amount);
    let result_amount = tax_info.subtract_tax(coin_amount);
    Ok(Coin {
        denom: coin.denom,
        amount: result_amount.into(),
    })
}

pub struct TaxInfo {
    pub rate: Decimal256,
    pub cap: Uint256,
}

impl TaxInfo {
    pub fn get_tax_for(&self, amount: Uint256) -> Uint256 {
        std::cmp::min(
            amount * (Decimal256::one() - Decimal256::one() / (Decimal256::one() + self.rate)),
            self.cap,
        )
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
