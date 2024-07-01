use near_sdk::base64;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum IntentError {
    #[error("Borsh")]
    Borsh,
    #[error("Base64: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("Intent '{0}' was not found")]
    NotFound(String), // id
    #[error("Intent '{0}' already exists")]
    AlreadyExists(String), // id
    #[error("Intent is already expired")]
    Expired,
    #[error("Intent is in progress")]
    InProgress,
    #[error("Wrong intent status")]
    WrongStatus,
    #[error("Amount mismatch")]
    AmountMismatch,
}
