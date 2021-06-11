use cosmwasm_std::{Coin, Deps, StdResult};

// pub mod asset;
pub mod basset_farmer;
pub mod basset_farmer_config;
pub mod overseer;
pub mod querier;

//TODO: move me
pub fn deduct_tax(deps: Deps, coin: Coin) -> StdResult<Coin> {
    todo!()
    // let tax_amount = compute_tax(deps, &coin)?;
    // Ok(Coin {
    //     denom: coin.denom,
    //     amount: (Uint256::from(coin.amount) - tax_amount).into(),
    // })
}
