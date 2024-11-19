use near_sdk::{serde_json, FunctionError};
use thiserror::Error as ThisError;

use crate::tokens::ParseTokenIdError;

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

    #[error("token_id: {0}")]
    ParseTokenId(#[from] ParseTokenIdError),

    #[error("wrong recipient")]
    WrongRecipient,

    #[error("zero amount(s)")]
    ZeroAmount,
}
