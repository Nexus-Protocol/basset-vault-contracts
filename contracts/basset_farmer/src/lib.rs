use cosmwasm_std::StdError;
use error::ContractError;
use std::convert::TryFrom;

mod commands;
pub mod contract;
pub mod error;
mod queries;
mod response;
pub mod state;
mod utils;

#[cfg(test)]
mod tests;

type ContractResult<T> = Result<T, ContractError>;

//withdrawing from Anchor Deposit error
pub const TOO_HIGH_BORROW_DEMAND_ERR_MSG: &str = "borrow demand too high";

pub enum SubmsgIds {
    InitNAssetConfigHolder,
    InitNAsset,
    InitNAssetRewards,
    InitPsiDistributor,
    RedeemStableOnRepayLoan,
    RepayLoan,
    Borrowing,
    RedeemStableOnRemainder,
}

impl TryFrom<u64> for SubmsgIds {
    type Error = ContractError;

    fn try_from(v: u64) -> Result<Self, Self::Error> {
        match v {
            x if x == SubmsgIds::InitNAssetConfigHolder.id() => {
                Ok(SubmsgIds::InitNAssetConfigHolder)
            }
            x if x == SubmsgIds::InitNAsset.id() => Ok(SubmsgIds::InitNAsset),
            x if x == SubmsgIds::InitNAssetRewards.id() => Ok(SubmsgIds::InitNAssetRewards),
            x if x == SubmsgIds::InitPsiDistributor.id() => Ok(SubmsgIds::InitPsiDistributor),
            x if x == SubmsgIds::RedeemStableOnRepayLoan.id() => {
                Ok(SubmsgIds::RedeemStableOnRepayLoan)
            }
            x if x == SubmsgIds::RepayLoan.id() => Ok(SubmsgIds::RepayLoan),
            x if x == SubmsgIds::Borrowing.id() => Ok(SubmsgIds::Borrowing),
            x if x == SubmsgIds::RedeemStableOnRemainder.id() => {
                Ok(SubmsgIds::RedeemStableOnRemainder)
            }
            unknown => {
                Err(StdError::generic_err(format!("unknown reply message id: {}", unknown)).into())
            }
        }
    }
}

impl SubmsgIds {
    pub const fn id(&self) -> u64 {
        match self {
            SubmsgIds::InitNAssetConfigHolder => 0,
            SubmsgIds::InitNAsset => 1,
            SubmsgIds::InitNAssetRewards => 2,
            SubmsgIds::InitPsiDistributor => 3,
            SubmsgIds::RedeemStableOnRepayLoan => 4,
            SubmsgIds::RepayLoan => 5,
            SubmsgIds::Borrowing => 6,
            SubmsgIds::RedeemStableOnRemainder => 7,
        }
    }
}
