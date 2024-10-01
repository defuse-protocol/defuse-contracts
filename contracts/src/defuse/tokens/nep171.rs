use near_contract_standards::non_fungible_token::{core::NonFungibleTokenReceiver, TokenId};
use near_sdk::{ext_contract, AccountId, PromiseOrValue};

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
