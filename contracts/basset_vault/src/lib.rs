use cosmwasm_std::StdError;
use std::convert::TryFrom;

mod commands;
pub mod contract;
mod queries;
mod reply_response;
pub mod state;
mod tax_querier;
mod utils;

#[cfg(test)]
mod tests;

//withdrawing from Anchor Deposit error
pub const TOO_HIGH_BORROW_DEMAND_ERR_MSG: &str = "borrow demand too high";
pub const MIN_ANC_REWARDS_TO_CLAIM: u64 = 100_000_000u64;
pub const MAX_SECS_DELAY_BETWEEN_ANC_CLAIM: u64 = 86_400;

pub enum SubmsgIds {
    InitNAssetConfigHolder,
    InitNAsset,
    InitNAssetPsiSwapPair,
    InitNAssetRewards,
    InitPsiDistributor,
    RedeemStableOnRepayLoan,
    RepayLoan,
    Borrowing,
    RedeemStableOnRemainder,
}

impl TryFrom<u64> for SubmsgIds {
    type Error = StdError;

    fn try_from(v: u64) -> Result<Self, Self::Error> {
        match v {
            x if x == SubmsgIds::InitNAssetConfigHolder.id() => {
                Ok(SubmsgIds::InitNAssetConfigHolder)
            }
            x if x == SubmsgIds::InitNAsset.id() => Ok(SubmsgIds::InitNAsset),
            x if x == SubmsgIds::InitNAssetPsiSwapPair.id() => Ok(SubmsgIds::InitNAssetPsiSwapPair),
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
            unknown => Err(StdError::generic_err(format!(
                "unknown reply message id: {}",
                unknown
            ))),
        }
    }
}

impl SubmsgIds {
    pub const fn id(&self) -> u64 {
        match self {
            SubmsgIds::InitNAssetConfigHolder => 0,
            SubmsgIds::InitNAsset => 1,
            SubmsgIds::InitNAssetPsiSwapPair => 2,
            SubmsgIds::InitNAssetRewards => 3,
            SubmsgIds::InitPsiDistributor => 4,
            SubmsgIds::RedeemStableOnRepayLoan => 5,
            SubmsgIds::RepayLoan => 6,
            SubmsgIds::Borrowing => 7,
            SubmsgIds::RedeemStableOnRemainder => 8,
        }
    }
}
