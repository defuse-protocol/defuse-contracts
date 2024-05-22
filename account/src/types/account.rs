use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};

#[derive(Default, Clone, BorshSerialize, BorshDeserialize, Deserialize, Serialize)]
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct Account {
    is_locked: bool,
}
