use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::{ext_contract, json_types::U128, serde::Serialize, AccountId, PromiseOrValue};

#[ext_contract(ext_ft_withdraw)]
pub trait FungibleTokenWithdrawer: FungibleTokenReceiver + FungibleTokenWithdrawResolver {
    /// Returns number of tokens were successfully withdrawn
    ///
    /// NOTE: MUST attach 1 yⓃ for security purposes.
    fn ft_withdraw(
        &mut self,
        token: AccountId,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: Option<String>,
    ) -> PromiseOrValue<U128>;
}

pub trait FungibleTokenWithdrawResolver {
    fn ft_resolve_withdraw(
        &mut self,
        token: AccountId,
        sender_id: AccountId,
        amount: U128,
        is_call: bool,
    ) -> U128;
}

#[must_use = "make sure to `.emit()` this event"]
#[derive(Debug, Serialize)]
#[serde(crate = "::near_sdk::serde")]
pub struct FtDepositEvent<'a> {
    pub sender_id: &'a AccountId,
    pub receiver_id: &'a AccountId,
    pub token: &'a AccountId,
    pub amount: U128,
}

#[must_use = "make sure to `.emit()` this event"]
#[derive(Debug, Serialize)]
#[serde(crate = "::near_sdk::serde")]
pub struct FtWithdrawEvent<'a> {
    pub sender_id: &'a AccountId,
    pub receiver_id: &'a AccountId,
    pub token: &'a AccountId,
    pub amount: U128,
}
