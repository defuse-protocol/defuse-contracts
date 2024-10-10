use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::{ext_contract, json_types::U128, AccountId, PromiseOrValue};

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
