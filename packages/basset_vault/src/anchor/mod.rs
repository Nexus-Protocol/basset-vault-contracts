pub mod basset_custody;
pub mod market;
pub mod borrow_apr;
pub mod earn_apr;

use bigint::U256;
use cosmwasm_bignumber::Decimal256;

/// 4656810
const NUMBER_OF_BLOCKS_PER_YEAR: Decimal256 = Decimal256(U256([3245568318532747264, 252446, 0, 0]));

#[cfg(test)]
#[test]
fn test_number_of_blocks_per_year_constant() {
    assert_eq!(NUMBER_OF_BLOCKS_PER_YEAR, Decimal256::from_uint256(4656810u64));
}
