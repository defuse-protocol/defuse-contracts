use near_sdk::serde_json;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum SwapIntentError {
    #[error("JSON: {0}")]
    JSON(serde_json::Error),
    #[error("intent with given ID not found")]
    NotFound,
    #[error("intent with given ID already exists")]
    AlreadyExists,
    #[error("intent has expired")]
    Expired,
    #[error("intent is still locked up")]
    LockedUp,
    #[error("wrong asset_out")]
    WrongAssetOut,
    #[error("wrong status")]
    WrongStatus,
    #[error("invalid recipient for given asset")]
    InvalidRecipient,
    #[error("unauthorized")]
    Unauthorized,
    #[error("zero amount")]
    ZeroAmount,
    #[error("intent id is too long")]
    IntentIdTooLong,
    #[error("lockup after expiration")]
    LockupAfterExpiration,
}
