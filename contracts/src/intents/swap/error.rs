use near_sdk::serde_json;
use thiserror::Error as ThisError;

use super::IntentId;

#[derive(Debug, ThisError)]
// TODO: rename
pub enum SwapError {
    #[error("JSON: {0}")]
    JSON(serde_json::Error),
    #[error("intent '{0}' not found")]
    NotFound(IntentId),
    #[error("intent '{0}' already exists")]
    AlreadyExists(IntentId),
    #[error("wrong asset")]
    // TODO: add expected/got
    WrongAsset,
    #[error("insufficient gas")]
    InsufficientGas,
    #[error("expired")]
    Expired,
    #[error("locked")]
    Locked,
    #[error("unlocked")]
    Unlocked,
    #[error("wrong status")]
    WrongStatus,
    #[error("zero amount")]
    ZeroAmount,
}
