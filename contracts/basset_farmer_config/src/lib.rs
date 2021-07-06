use error::ContractError;

mod commands;
pub mod contract;
pub mod error;
mod price;
mod queries;
pub mod state;

#[cfg(test)]
#[allow(dead_code)]
mod tests;

type ContractResult<T> = Result<T, ContractError>;
