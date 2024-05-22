use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::AccountId;
use nep141::Nep141;

pub mod nep141;

#[derive(BorshDeserialize, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
pub struct Intent {
    pub owner: AccountId,
    pub intent_type: IntentType,
    pub status: Status,
}

impl Intent {
    pub fn new(owner: AccountId, intent_type: IntentType) -> Self {
        Self {
            owner,
            intent_type,
            status: Status::CreatedByUser,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(untagged, crate = "near_sdk::serde")]
pub enum IntentType {
    Nep141(Nep141),
}

#[derive(Default, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
pub enum Status {
    #[default]
    CreatedByUser,
    ApprovedBySolver,
    Expired,
}
