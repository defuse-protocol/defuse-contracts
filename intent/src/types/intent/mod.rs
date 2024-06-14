use near_sdk::json_types::U128;
use near_sdk::{
    base64::{engine::general_purpose::STANDARD, Engine},
    borsh::BorshDeserialize,
    env, near, AccountId,
};

use crate::error::ContractError;

/// Intent with status
#[derive(Debug)]
#[near(serializers=[borsh, json])]
pub struct DetailedIntent {
    /// Intent
    pub intent: Intent,
    /// Status of the intent
    pub status: Status,
    /// Time of the creation
    pub created_at: u64,
    /// Minimum TTL of the intent
    pub min_ttl: u64,
}

impl DetailedIntent {
    /// Create a new intent with status `Created`
    #[must_use]
    pub fn new(intent: Intent, min_ttl: u64) -> Self {
        Self {
            intent,
            status: Status::Available,
            created_at: env::block_timestamp_ms(),
            min_ttl: min_ttl * 1000, // Safe. Validation for overflow happens in the transaction
        }
    }

    /// Get the inner intent
    #[must_use]
    pub const fn get_intent(&self) -> &Intent {
        &self.intent
    }

    /// Get the status of the intent
    #[must_use]
    pub const fn get_status(&self) -> Status {
        self.status
    }

    /// Set the status of the intent
    pub fn set_status(&mut self, status: Status) {
        self.status = status;
    }

    /// Check if the intent could be rolled back
    #[must_use]
    pub fn could_be_rollbacked(&self) -> bool {
        env::block_timestamp_ms() - self.created_at > self.min_ttl
    }
}

/// Intent for swapping NEP-141 tokens
#[derive(Debug)]
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
    /// Check if the intent is expired
    #[must_use]
    pub fn is_expired(&self) -> bool {
        match self.expiration {
            Expiration::None => false,
            Expiration::Time(time) => time * 1000 <= env::block_timestamp_ms(),
            Expiration::Block(block) => block <= env::block_height(),
        }
    }
}

#[near(serializers=[borsh])]
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
    pub fn decode<M: AsRef<str>>(msg: M) -> Result<Self, ContractError> {
        let bytes = STANDARD
            .decode(msg.as_ref())
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

/// The struct describes the token and amount of these tokens a user wants to exchange
#[derive(Debug, Clone)]
#[near(serializers=[borsh, json])]
pub struct TokenAmount {
    /// Token account id
    pub token_id: AccountId,
    /// Amount of tokens
    pub amount: U128,
}

/// Intent status
#[derive(Debug, Clone, Copy)]
#[near(serializers=[borsh, json])]
pub enum Status {
    /// Intent is created by the user and available for the execution
    Available,
    /// Intent is processing
    Processing,
    /// Intent is completed by the solver
    Completed,
    /// Intent is rolled back by the user or owner
    RolledBack,
    /// Intent is expired
    Expired,
}

#[test]
fn test_create_action_serialize() {
    let action = Action::CreateIntent((
        "1".to_string(),
        Intent {
            initiator: "user.near".parse().unwrap(),
            send: TokenAmount {
                token_id: "token_a.near".parse().unwrap(),
                amount: 1000.into(),
            },
            receive: TokenAmount {
                token_id: "token_b.near".parse().unwrap(),
                amount: 2000.into(),
            },
            expiration: Expiration::Block(123_456),
            referral: Some("referral.near".parse().unwrap()),
        },
    ));

    assert_eq!(
        action.encode().unwrap(),
        "AAEAAAAxCQAAAHVzZXIubmVhcgwAAAB0b2tlbl9hLm5lYXLoAwAAAAAAAAAAAAAAAAAADAAAAHRva2VuX2IubmVhctAHAAAAAAAAAAAAAAAAAAACQOIBAAAAAAABDQAAAHJlZmVycmFsLm5lYXI="
    );
}

#[test]
fn test_create_action_deserialize() {
    let action = Action::decode("AAEAAAAxCQAAAHVzZXIubmVhcgwAAAB0b2tlbl9hLm5lYXLoAwAAAAAAAAAAAAAAAAAADAAAAHRva2VuX2IubmVhctAHAAAAAAAAAAAAAAAAAAACQOIBAAAAAAABDQAAAHJlZmVycmFsLm5lYXI=").unwrap();
    assert!(matches!(action, Action::CreateIntent((id, _)) if id == "1"));
}

#[test]
fn test_execute_action_serialize() {
    let action = Action::ExecuteIntent("1".to_string());
    assert_eq!(action.encode().unwrap(), "AQEAAAAx");
}

#[test]
fn test_execute_action_deserialize() {
    let action = Action::decode("AQEAAAAx").unwrap();
    assert!(matches!(action, Action::ExecuteIntent(id) if id == "1"));
}
