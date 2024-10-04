use near_contract_standards::non_fungible_token;
use near_sdk::{near, AccountId, Gas};
use serde_with::{serde_as, DisplayFromStr};

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

#[cfg_attr(
    all(feature = "abi", not(target_arch = "wasm32")),
    serde_as(schemars = true)
)]
#[cfg_attr(
    not(all(feature = "abi", not(target_arch = "wasm32"))),
    serde_as(schemars = false)
)]
#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct Nep141Withdraw {
    pub token: AccountId,
    pub receiver_id: AccountId,
    #[serde_as(as = "DisplayFromStr")]
    pub amount: u128,
    pub memo: Option<String>,
    pub msg: Option<String>,

    // TODO
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

    // TODO
    pub gas: Option<Gas>,
}

#[cfg_attr(
    all(feature = "abi", not(target_arch = "wasm32")),
    serde_as(schemars = true)
)]
#[cfg_attr(
    not(all(feature = "abi", not(target_arch = "wasm32"))),
    serde_as(schemars = false)
)]
#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct Nep245Withdraw {
    pub token: AccountId,
    pub receiver_id: AccountId,
    #[serde_as(as = "Vec<(_, DisplayFromStr)>")]
    pub amounts: Vec<(nep245::TokenId, u128)>,
    pub memo: Option<String>,
    pub msg: Option<String>,

    // TODO
    pub gas: Option<Gas>,
}
