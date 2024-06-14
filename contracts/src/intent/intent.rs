use near_sdk::{
    base64::engine::{general_purpose::STANDARD, Engine},
    borsh::{self, BorshDeserialize},
    env,
    json_types::U128,
    near, AccountId,
};

use super::IntentError;

#[near(serializers=[borsh])]
pub enum Action {
    CreateIntent(
        /// ID
        String,
        Intent,
    ),
    ExecuteIntent(
        /// ID
        String,
    ),
}

impl Action {
    pub fn base64_decode(msg: impl AsRef<[u8]>) -> Result<Self, IntentError> {
        Self::try_from_slice(&STANDARD.decode(msg)?).map_err(|_| IntentError::Borsh)
    }

    pub fn encode_base64(&self) -> Result<String, IntentError> {
        Ok(STANDARD.encode(borsh::to_vec(self).map_err(|_| IntentError::Borsh)?))
    }
}

/// Intent for swapping NEP-141 tokens
#[near(serializers=[borsh, json])]
pub struct Intent {
    /// Initiator of the intent
    pub initiator: AccountId,
    /// Tokens the initiator wants to exchange
    pub send: TokenAmount,
    /// Tokens the initiator wants to get instead
    pub receive: TokenAmount,
    /// Intent expiration
    pub expiration: Expiration,
    /// Referral for getting a fee
    pub referral: Option<AccountId>,
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

/// Intent expiration
#[derive(Default, Debug)]
#[near(serializers=[borsh, json])]
pub enum Expiration {
    /// No expiration
    #[default]
    None,
    /// Expiration time in seconds.
    Time(u64),
    /// Expiration block.
    Block(u64),
}

#[derive(Debug, Clone)]
#[near(serializers=[borsh, json])]
pub struct TokenAmount {
    pub token_id: AccountId,
    pub amount: U128,
}
