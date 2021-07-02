use error::ContractError;

pub mod contract;
pub mod error;

#[cfg(test)]
#[allow(dead_code)]
mod tests;

type ContractResult<T> = Result<T, ContractError>;
