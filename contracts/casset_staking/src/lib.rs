use error::ContractError;

mod commands;
pub mod contract;
pub mod error;
pub mod state;

type ContractResult<T> = Result<T, ContractError>;
