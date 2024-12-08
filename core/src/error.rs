use near_sdk::{serde_json, FunctionError};
use thiserror::Error as ThisError;

use crate::{intents::token_diff::TokenDeltas, tokens::ParseTokenIdError};

pub type Result<T, E = DefuseError> = ::core::result::Result<T, E>;

#[derive(Debug, ThisError, FunctionError)]
pub enum DefuseError {
    #[error("account not found")]
    AccountNotFound,

    #[error("insufficient balance or overflow")]
    BalanceOverflow,

    #[error("deadline has expired")]
    DeadlineExpired,

    #[error("invalid intent")]
    InvalidIntent,

    #[error("invalid signature")]
    InvalidSignature,

    #[error(
        "invariant violated, unmatched deltas{}",
        .0.as_ref()
            .map(|v| {
                format!(
                    ": {}",
                    serde_json::to_string(v).unwrap_or_else(|_| unreachable!()),
                )
            })
            .unwrap_or_default()
    )]
    UnmatchedDeltas(Option<TokenDeltas>),

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

    #[error("wrong verifying_contract")]
    WrongVerifyingContract,
}
