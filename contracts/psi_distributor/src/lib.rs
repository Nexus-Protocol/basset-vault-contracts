use error::ContractError;

mod commands;
pub mod contract;
pub mod error;
pub mod queries;
pub mod state;
pub mod utils;

// #[cfg(test)]
// mod tests;

type ContractResult<T> = Result<T, ContractError>;
