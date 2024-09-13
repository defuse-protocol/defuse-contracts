use near_sdk::{json_types::U128, AccountId, Promise};

pub trait FungibleTokenWithdrawer {
    /// Returns whether the tokens were successfully withdrawn
    fn nep141_withdraw(
        &mut self,
        token_id: AccountId,
        to: Option<AccountId>,
        amount: U128,
    ) -> Promise;

    /// Returns number of tokens were successfully withdrawn
    fn nep141_withdraw_call(
        &mut self,
        token_id: AccountId,
        to: Option<AccountId>,
        amount: U128,
        msg: String,
    ) -> Promise;
}
