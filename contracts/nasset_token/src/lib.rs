use error::ContractError;

mod commands;
pub mod contract;
pub mod error;
pub mod querier;
pub mod state;

#[cfg(test)]
#[allow(dead_code)]
mod tests;

type ContractResult<T> = Result<T, ContractError>;
