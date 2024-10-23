use defuse_contracts::{
    defuse::{
        intents::{ext_intents_executor, IntentsExecutor},
        tokens::{DepositMessage, TokenId},
    },
    utils::{
        cache::{CURRENT_ACCOUNT_ID, PREDECESSOR_ACCOUNT_ID},
        UnwrapOrPanic, UnwrapOrPanicError,
    },
};
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_plugins::{pause, Pausable};
use near_sdk::{json_types::U128, near, require, AccountId, PromiseOrValue};

use crate::{DefuseImpl, DefuseImplExt};
#[near]
impl FungibleTokenReceiver for DefuseImpl {
    /// Deposit fungible tokens.
    ///
    /// `msg` contains [`AccountId`] of the internal recipient.
    /// Empty `msg` means deposit to `sender_id`
    #[pause]
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        require!(amount.0 > 0, "zero amount");

        let msg = if !msg.is_empty() {
            msg.parse().unwrap_or_panic_display()
        } else {
            DepositMessage::new(sender_id)
        };

        self.internal_deposit(
            msg.receiver_id,
            [(TokenId::Nep141(PREDECESSOR_ACCOUNT_ID.clone()), amount.0)],
            Some("deposit"),
        )
        .unwrap_or_panic();

        if !msg.execute_intents.is_empty() {
            if msg.refund_if_fails {
                self.execute_intents(msg.execute_intents);
            } else {
                // detach promise
                ext_intents_executor::ext(CURRENT_ACCOUNT_ID.clone())
                    .execute_intents(msg.execute_intents);
            }
        }

        PromiseOrValue::Value(U128(0))
    }
}
