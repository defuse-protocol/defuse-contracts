use near_sdk::{ext_contract, json_types::U128, AccountId, PromiseOrValue};

use crate::nep245::{receiver::MultiTokenReceiver, TokenId};

#[ext_contract(ext_mt_withdraw)]
pub trait MultiTokenWithdrawer: MultiTokenReceiver + MultiTokenWithdrawResolver {
    /// Returns number of tokens were successfully withdrawn
    ///
    /// Optionally can specify `storage_deposit` for `receiver_id` on `token`.
    /// The amount will be subtracted from user's NEP-141 `wNEAR` balance.
    ///
    /// NOTE: MUST attach 1 yⓃ for security purposes.
    fn mt_withdraw(
        &mut self,
        token: AccountId,
        receiver_id: AccountId,
        token_ids: Vec<TokenId>,
        amounts: Vec<U128>,
        memo: Option<String>,
    ) -> PromiseOrValue<bool>;
}

#[ext_contract(mt_withdraw_resolver)]
pub trait MultiTokenWithdrawResolver {
    fn mt_resolve_withdraw(
        &mut self,
        token: AccountId,
        sender_id: AccountId,
        token_ids: Vec<TokenId>,
        amounts: Vec<U128>,
    ) -> bool;
}
