use core::convert::Infallible;

use near_sdk::{serde_json, FunctionError};
use thiserror::Error as ThisError;

pub type Result<T, E = DefuseError> = ::core::result::Result<T, E>;

#[derive(Debug, ThisError, FunctionError)]
pub enum DefuseError {
    #[error("account not found")]
    AccountNotFound,

    #[error("insufficient or excessive balance")]
    BalanceOverflow,

    #[error("deadline has expired")]
    DeadlineExpired,

    #[error("invalid sender/receiver")]
    InvalidSenderReceiver,

    #[error("invalid signature")]
    InvalidSignature,

    #[error("invariant violated")]
    InvariantViolated,

    #[error("JSON: {0}")]
    JSON(#[from] serde_json::Error),

    #[error("nonce was already used")]
    NonceUsed,

    #[error("wrong recipient")]
    WrongRecipient,
}

// Remove when `!` is stabilized:
// https://github.com/rust-lang/rust/issues/35121
impl From<Infallible> for DefuseError {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}
