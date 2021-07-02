use error::ContractError;

mod commands;
pub mod contract;
pub mod error;
mod math;
mod queries;
pub mod state;
mod utils;

#[cfg(test)]
#[allow(dead_code)]
mod tests;

type ContractResult<T> = Result<T, ContractError>;
