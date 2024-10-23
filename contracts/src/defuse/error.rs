use core::convert::Infallible;

use near_sdk::{serde_json, FunctionError};
use thiserror::Error as ThisError;

pub type Result<T, E = DefuseError> = ::core::result::Result<T, E>;

#[derive(Debug, ThisError, FunctionError)]
pub enum DefuseError {
    #[error("account not found")]
    AccountNotFound,

    #[error("insufficient balance or overflow")]
    BalanceOverflow,

    #[error("deadline has expired")]
    DeadlineExpired,

    #[error("invalid signature")]
    InvalidSignature,

    #[error("invariant violated")]
    InvariantViolated,

    #[error("JSON: {0}")]
    JSON(#[from] serde_json::Error),

    #[error("nonce was already used")]
    NonceUsed,

    #[error("public key already exists")]
    PublicKeyExists,

    #[error("public key doesn't exist")]
    PublicKeyNotExist,

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
