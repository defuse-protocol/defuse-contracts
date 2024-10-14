use near_contract_standards::non_fungible_token::{core::NonFungibleTokenReceiver, TokenId};
use near_sdk::{ext_contract, serde::Serialize, AccountId, PromiseOrValue};

#[ext_contract(ext_nft_withdraw)]
pub trait NonFungibleTokenWithdrawer:
    NonFungibleTokenReceiver + NonFungibleTokenWithdrawResolver
{
    /// Returns number of tokens were successfully withdrawn
    ///
    /// NOTE: MUST attach 1 yⓃ for security purposes.
    fn nft_withdraw(
        &mut self,
        token: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        memo: Option<String>,
        msg: Option<String>,
    ) -> PromiseOrValue<bool>;
}

pub trait NonFungibleTokenWithdrawResolver {
    fn nft_resolve_withdraw(
        &mut self,
        token: AccountId,
        sender_id: AccountId,
        token_id: TokenId,
        is_call: bool,
    ) -> bool;
}

#[must_use = "make sure to `.emit()` this event"]
#[derive(Debug, Serialize)]
#[serde(crate = "::near_sdk::serde")]
pub struct NftDepositEvent<'a> {
    pub sender_id: &'a AccountId,
    pub receiver_id: &'a AccountId,
    pub token: &'a AccountId,
    pub token_id: &'a TokenId,
}

#[must_use = "make sure to `.emit()` this event"]
#[derive(Debug, Serialize)]
#[serde(crate = "::near_sdk::serde")]
pub struct NftWithdrawEvent<'a> {
    pub sender_id: &'a AccountId,
    pub receiver_id: &'a AccountId,
    pub token: &'a AccountId,
    pub token_id: &'a TokenId,
}
