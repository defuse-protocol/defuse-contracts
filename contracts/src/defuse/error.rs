use near_sdk::serde_json;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum DefuseError {
    #[error("insufficient or excessive balance")]
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

    #[error("wrong recipient")]
    WrongRecipient,
}
