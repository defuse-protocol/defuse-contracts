use near_contract_standards::non_fungible_token::core::NonFungibleTokenReceiver;
use near_sdk::{
    base64::{engine::general_purpose::STANDARD, Engine},
    borsh::{self, BorshDeserialize},
    near, AccountId, Promise,
};

pub use self::error::*;

mod error;

pub trait MpcIntentContract: NonFungibleTokenReceiver {
    fn rollback_intent(&mut self, id: IntentID) -> Promise;
}

#[derive(Debug, PartialEq, Eq)]
#[near(serializers=[borsh, json])]
pub struct Account {
    pub account_shard: AccountId,
    pub derivation_path: String,
}

#[derive(Debug)]
#[near(serializers=[borsh, json])]
pub enum Intent {
    Barter(BarterIntent),
}

/// Swap two specified accounts
#[derive(Debug)]
#[near(serializers=[borsh, json])]
pub struct BarterIntent {
    // TODO: store
    // pub send: Account,
    pub receive: Account,
    // Typically, sender
    // TODO: make optional
    pub recepient: AccountId,
    pub memo: Option<String>,
    // TODO: to forward msg
    pub msg: Option<String>,
}

pub type IntentID = String;

#[near(serializers = [borsh])]
pub enum Action {
    Create {
        // TODO: maybe generate it on-chain?
        id: IntentID,
        intent: Intent,
    },
    Fulfill {
        id: IntentID,
        /// By default: back to sender
        recipient: Option<AccountId>,
        memo: Option<String>,
        msg: Option<String>,
    },
}

impl Action {
    pub fn decode(msg: impl AsRef<[u8]>) -> Result<Self, MpcIntentError> {
        Self::try_from_slice(&STANDARD.decode(msg)?).map_err(|_| MpcIntentError::Borsh)
    }

    pub fn encode(&self) -> Result<String, MpcIntentError> {
        Ok(STANDARD.encode(borsh::to_vec(self).map_err(|_| MpcIntentError::Borsh)?))
    }
}

#[derive(Debug)]
#[near(serializers=[borsh, json])]
pub struct RegisteredIntent {
    pub send: Account,
    pub intent: Intent,
}
