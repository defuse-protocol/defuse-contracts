use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    base64::{engine::general_purpose::STANDARD, Engine},
    borsh::{BorshDeserialize, BorshSerialize},
    env, AccountId,
};

use crate::error::ContractError;

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct Intent {
    pub initiator: AccountId,
    pub send: TokenAmount,
    pub receive: TokenAmount,
    pub expiration: Expiration,
}

impl Intent {
    #[must_use]
    pub fn is_expired(&self) -> bool {
        match self.expiration {
            Expiration::None => false,
            Expiration::Time(time) => time * 1000 <= env::block_timestamp_ms(),
            Expiration::Block(block) => block <= env::block_height(),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
pub enum Action {
    CreateIntent((String, Intent)),
    ExecuteIntent(String),
}

impl Action {
    /// Decode provided msg into `Action`.
    ///
    /// # Errors
    ///
    /// `Base64DecodeError`
    /// `BorshDeserializeError`
    pub fn decode(msg: &str) -> Result<Self, ContractError> {
        let bytes = STANDARD
            .decode(msg)
            .map_err(|_| ContractError::Base64DecodeError)?;

        Self::try_from_slice(&bytes).map_err(|_| ContractError::BorshDeserializeError)
    }

    /// Encode the action into a string.
    ///
    /// # Errors
    ///
    /// `BorshSerializeError`
    pub fn encode(&self) -> Result<String, ContractError> {
        near_sdk::borsh::to_vec(&self)
            .map(|bytes| STANDARD.encode(bytes))
            .map_err(|_| ContractError::BorshSerializeError)
    }
}

#[derive(Default, Debug, BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub enum Expiration {
    #[default]
    None,
    /// Expiration time in seconds.
    Time(u64),
    /// Expiration block.
    Block(u64),
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct TokenAmount {
    pub token_id: AccountId,
    pub amount: U128,
}
