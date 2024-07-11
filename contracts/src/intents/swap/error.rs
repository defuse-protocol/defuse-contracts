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
    #[error("wrong asset or amount")]
    WrongAssetOrAmount,
    #[error("wrong status")]
    WrongStatus,
    #[error("unauthorized")]
    Unauthorized,
    #[error("zero amount")]
    ZeroAmount,
}
