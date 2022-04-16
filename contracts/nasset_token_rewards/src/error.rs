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

//need it only for 'query_holder' function
impl From<ContractError> for StdError {
    fn from(err: ContractError) -> Self {
        match err {
            ContractError::Std(std) => std,
            ContractError::Unauthorized => StdError::generic_err("unauthorized"),
            ContractError::Impossible(msg) => {
                StdError::generic_err(format!("impossible case, message: '{}'", msg))
            }
            ContractError::Overflow { .. } => StdError::generic_err("calculations overflow"),
        }
    }
}
