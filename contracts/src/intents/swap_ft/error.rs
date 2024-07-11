use near_sdk::base64;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum FtIntentError {
    #[error("borsh deserialize error")]
    BorshDeserialize,
    #[error("borsh serialize error")]
    BorshSerialize,
    #[error("base64: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("intent '{0}' was not found")]
    NotFound(String), // id
    #[error("intent '{0}' already exists")]
    AlreadyExists(String), // id
    #[error("intent is already expired")]
    Expired,
    #[error("intent is in progress")]
    InProgress,
    #[error("wrong intent status")]
    WrongStatus,
    #[error("amount mismatch")]
    AmountMismatch,
}
