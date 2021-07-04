use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Impossible: {0}")]
    Impossible(String),

    #[error("Overflow: {source}")]
    Overflow {
        source: OverflowError,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
}

impl ContractError {
    pub fn overflow(source: OverflowError) -> Self {
        ContractError::Overflow {
            source,
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }
}

impl From<OverflowError> for ContractError {
    fn from(source: OverflowError) -> Self {
        Self::overflow(source)
    }
}
