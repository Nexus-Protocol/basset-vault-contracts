use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Coin, Deps, StdResult};
use terra_cosmwasm::TerraQuerier;

// pub mod asset;
pub mod basset_farmer;
pub mod basset_farmer_config;
pub mod overseer;
pub mod querier;

pub fn deduct_tax(deps: Deps, coin: Coin) -> StdResult<Coin> {
    let tax_info = get_tax_info(deps, &coin.denom)?;
    let coin_amount = Uint256::from(coin.amount);
    let result_amount = subtract_tax(coin_amount, &tax_info);
    Ok(Coin {
        denom: coin.denom,
        amount: result_amount.into(),
    })
}

pub struct TaxInfo {
    pub rate: Decimal256,
    pub cap: Uint256,
}

pub fn get_tax_info(deps: Deps, coin_denom: &str) -> StdResult<TaxInfo> {
    let terra_querier = TerraQuerier::new(&deps.querier);
    let rate = Decimal256::from((terra_querier.query_tax_rate()?).rate);
    let cap = Uint256::from((terra_querier.query_tax_cap(coin_denom)?).cap);
    return Ok(TaxInfo { rate, cap });
}

pub fn subtract_tax(coin_amount: Uint256, tax: &TaxInfo) -> Uint256 {
    let tax_amount = std::cmp::min(
        coin_amount * (Decimal256::one() - Decimal256::one() / (Decimal256::one() + tax.rate)),
        tax.cap,
    );
    coin_amount - tax_amount
}
