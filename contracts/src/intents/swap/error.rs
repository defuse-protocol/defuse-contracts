use near_sdk::serde_json;
use thiserror::Error as ThisError;

use super::IntentID;

#[derive(Debug, ThisError)]
pub enum SwapError {
    #[error("JSON: {0}")]
    JSON(serde_json::Error),
    #[error("intent '{0}' not found")]
    NotFound(IntentID),
    #[error("intent '{0}' already exists")]
    AlreadyExists(IntentID),
    #[error("wrong asset")]
    // TODO: add expected/got
    WrongAsset,
    #[error("insufficient gas")]
    InsufficientGas,
    #[error("expired")]
    Expired,
    #[error("locked")]
    Locked,
}
