use defuse_contracts::{
    defuse::tokens::TokenId,
    utils::{cache::PREDECESSOR_ACCOUNT_ID, UnwrapOrPanic},
};
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::{json_types::U128, near, AccountId, PromiseOrValue};

use crate::{DefuseImpl, DefuseImplExt};
#[near]
impl FungibleTokenReceiver for DefuseImpl {
    /// Deposit fungible tokens.
    ///
    /// `msg` contains [`AccountId`] of the internal recipient.
    /// Empty `msg` means deposit to `sender_id`
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let receiver_id = if !msg.is_empty() {
            msg.parse().unwrap_or_panic_display()
        } else {
            sender_id
        };

        self.internal_deposit(
            receiver_id,
            [(TokenId::Nep141(PREDECESSOR_ACCOUNT_ID.clone()), amount.0)],
        )
        .unwrap_or_panic();

        PromiseOrValue::Value(U128(0))
    }
}
