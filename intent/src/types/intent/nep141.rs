use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::AccountId;

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct Nep141 {
    pub output: TokenAmount,
    pub input: TokenAmount,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct TokenAmount {
    pub token_id: AccountId,
    pub amount: U128,
}
