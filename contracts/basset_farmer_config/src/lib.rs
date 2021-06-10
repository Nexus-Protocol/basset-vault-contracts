use error::ContractError;

mod commands;
pub mod contract;
pub mod error;
mod queries;
pub mod state;

#[cfg(test)]
mod tests;

type ContractResult<T> = Result<T, ContractError>;
