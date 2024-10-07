use near_contract_standards::non_fungible_token;
use near_sdk::json_types::U128;
use near_sdk::{near, AccountId, Gas};

use crate::nep245;

use crate::defuse::tokens::TokenAmounts;

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct TokenTransfer {
    pub recipient_id: AccountId,
    pub tokens: TokenAmounts<u128>,
    // TODO: memo
}

#[near(serializers = [borsh, json])]
#[serde(tag = "token_standard", rename_all = "snake_case")]
#[derive(Debug, Clone)]
pub enum TokenWithdraw {
    Nep141(Nep141Withdraw),
    Nep171(Nep171Withdraw),
    Nep245(Nep245Withdraw),
}

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct Nep141Withdraw {
    pub token: AccountId,
    pub receiver_id: AccountId,
    pub amount: U128,
    pub memo: Option<String>,
    pub msg: Option<String>,
    pub gas: Option<Gas>,
}

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct Nep171Withdraw {
    pub token: AccountId,
    pub receiver_id: AccountId,
    pub token_id: non_fungible_token::TokenId,
    pub memo: Option<String>,
    pub msg: Option<String>,
    pub gas: Option<Gas>,
}

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct Nep245Withdraw {
    pub token: AccountId,
    pub receiver_id: AccountId,
    pub token_id_amounts: Vec<(nep245::TokenId, U128)>,
    pub memo: Option<String>,
    pub msg: Option<String>,
    pub gas: Option<Gas>,
}
