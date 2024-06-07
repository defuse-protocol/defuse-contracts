use near_sdk::json_types::U128;
use near_sdk::{
    base64::{engine::general_purpose::STANDARD, Engine},
    borsh::BorshDeserialize,
    env, near, AccountId,
};

use crate::error::ContractError;

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
