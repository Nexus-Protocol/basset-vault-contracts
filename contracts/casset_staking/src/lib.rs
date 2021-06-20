use error::ContractError;

mod commands;
pub mod contract;
pub mod error;
pub mod queries;
pub mod state;
pub mod utils;

type ContractResult<T> = Result<T, ContractError>;
