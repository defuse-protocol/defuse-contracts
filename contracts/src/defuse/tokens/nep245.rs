use near_sdk::{ext_contract, json_types::U128, serde::Serialize, AccountId, PromiseOrValue};
use serde_with::{serde_as, DisplayFromStr};

use crate::nep245::{receiver::MultiTokenReceiver, TokenId};

#[ext_contract(ext_mt_withdraw)]
pub trait MultiTokenWithdrawer: MultiTokenReceiver + MultiTokenWithdrawResolver {
    /// Returns number of tokens were successfully withdrawn
    ///
    /// NOTE: MUST attach 1 yⓃ for security purposes.
    fn mt_withdraw(
        &mut self,
        token: AccountId,
        receiver_id: AccountId,
        token_ids: Vec<TokenId>,
        amounts: Vec<U128>,
        memo: Option<String>,
        msg: Option<String>,
    ) -> PromiseOrValue<Vec<U128>>;
}

pub trait MultiTokenWithdrawResolver {
    fn mt_resolve_withdraw(
        &mut self,
        token: AccountId,
        sender_id: AccountId,
        token_ids: Vec<TokenId>,
        amounts: Vec<U128>,
        is_call: bool,
    ) -> Vec<U128>;
}

#[must_use = "make sure to `.emit()` this event"]
#[serde_as]
#[derive(Debug, Serialize)]
#[serde(crate = "::near_sdk::serde")]
pub struct FtDepositEvent<'a> {
    pub sender_id: &'a AccountId,
    pub receiver_id: &'a AccountId,
    pub token: &'a AccountId,
    #[serde_as(as = "DisplayFromStr")]
    pub amount: u128,
}

#[must_use = "make sure to `.emit()` this event"]
#[serde_as]
#[derive(Debug, Serialize)]
#[serde(crate = "::near_sdk::serde")]
pub struct FtWithdrawEvent<'a> {
    pub sender_id: &'a AccountId,
    pub receiver_id: &'a AccountId,
    pub token: &'a AccountId,
    #[serde_as(as = "DisplayFromStr")]
    pub amount: u128,
}
