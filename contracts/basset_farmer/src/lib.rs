use error::ContractError;

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
