use near_sdk::base64;
use thiserror::Error as ThisError;

use super::IntentID;

#[derive(Debug, ThisError)]
pub enum MpcIntentError {
    #[error("Borsh")]
    Borsh,
    #[error("base64: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("intent '{0}' was not found")]
    NotFound(IntentID),
    #[error("intent '{0}' already exists")]
    AlreadyExists(IntentID),
    // #[error("intent has been already expired")]
    // Expired,
    #[error("intent is in progress")]
    InProgress,
    #[error("wrong status")]
    WrongStatus,
    #[error("account mismatch")]
    WrongAccount,
}
