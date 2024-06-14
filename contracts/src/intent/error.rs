use near_sdk::base64;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum IntentError {
    #[error("Borsh")]
    Borsh,
    #[error("base64: {0}")]
    Base64(#[from] base64::DecodeError),
}
