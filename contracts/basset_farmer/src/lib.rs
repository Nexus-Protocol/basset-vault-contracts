use error::ContractError;

mod commands;
pub mod contract;
pub mod error;
mod libs;
mod queries;
mod response;
pub mod state;
mod utils;

#[cfg(test)]
mod tests;

// #[cfg(test)]
// mod mock_querier;

type ContractResult<T> = Result<T, ContractError>;
