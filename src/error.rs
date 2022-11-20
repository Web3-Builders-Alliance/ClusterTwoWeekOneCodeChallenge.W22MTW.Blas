use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Contract initialization. A coin greater than 1 must be provided to be able to Like Messages.")]
    ValidCoinRequired {},

    #[error("Message ID not valid: Not one message instance found")]
    InvalidMessageID{},

    #[error("Invalid funds. {val2:?} {val1:?} must be transferred when creating message")]
    InvalidFundsMessage{val1: String, val2: String},

    #[error("Invalid funds. {val2:?} {val1:?} must be transferred when liking message")]
    InvalidFundsLike{val1: String, val2: String},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
