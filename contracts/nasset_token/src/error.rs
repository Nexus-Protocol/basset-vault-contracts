use cosmwasm_std::StdError;
use cw20_base::ContractError as Cw20ContractError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Cannot set to own account")]
    CannotSetOwnAccount,

    #[error("Invalid zero amount")]
    InvalidZeroAmount,

    #[error("Allowance is expired")]
    Expired,

    #[error("No allowance for this account")]
    NoAllowance,

    #[error("Minting cannot exceed the cap")]
    CannotExceedCap,

    #[error("Logo too big")]
    LogoTooBig,

    #[error("Invalid xml preamble for SVG")]
    InvalidXmlPreamble,

    #[error("Invalid png header")]
    InvalidPngHeader,
}

impl From<Cw20ContractError> for ContractError {
    fn from(cw20_error: Cw20ContractError) -> Self {
        match cw20_error {
            Cw20ContractError::Std(std_error) => ContractError::Std(std_error),
            Cw20ContractError::Unauthorized {} => ContractError::Unauthorized,
            Cw20ContractError::CannotSetOwnAccount {} => ContractError::CannotSetOwnAccount,
            Cw20ContractError::InvalidZeroAmount {} => ContractError::InvalidZeroAmount,
            Cw20ContractError::Expired {} => ContractError::Expired,
            Cw20ContractError::NoAllowance {} => ContractError::NoAllowance,
            Cw20ContractError::CannotExceedCap {} => ContractError::CannotExceedCap,
            Cw20ContractError::LogoTooBig {} => ContractError::LogoTooBig,
            Cw20ContractError::InvalidXmlPreamble {} => ContractError::InvalidXmlPreamble,
            Cw20ContractError::InvalidPngHeader {} => ContractError::InvalidPngHeader,
        }
    }
}
