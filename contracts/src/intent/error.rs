use near_sdk::base64;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum IntentError {
    #[error("Borsh")]
    Borsh,
    #[error("base64: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("intent '{0}' was not found")]
    NotFound(
        /// ID
        String,
    ),
    #[error("intent '{0}' already exists")]
    AlreadyExists(
        /// ID
        String,
    ),
    #[error("intent has been already expired")]
    Expired,
    #[error("intent is in progress")]
    InProgress,
    #[error("wrong status")]
    WrongStatus,
    #[error("amount mismatch")]
    AmountMismatch,
}
