use defuse_contracts::{
    defuse::{
        intents::{ext_intents_executor, IntentsExecutor},
        tokens::{DepositMessage, TokenId},
    },
    nep245::receiver::MultiTokenReceiver,
    utils::{
        cache::{CURRENT_ACCOUNT_ID, PREDECESSOR_ACCOUNT_ID},
        UnwrapOrPanic, UnwrapOrPanicError,
    },
};
use near_plugins::{pause, Pausable};
use near_sdk::{json_types::U128, near, require, AccountId, PromiseOrValue};

use crate::{DefuseImpl, DefuseImplExt};

#[near]
impl MultiTokenReceiver for DefuseImpl {
    /// Deposit multi-tokens.
    ///
    /// `msg` contains [`AccountId`] of the internal recipient.
    /// Empty `msg` means deposit to `sender_id`
    #[pause]
    fn mt_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_ids: Vec<AccountId>,
        token_ids: Vec<defuse_contracts::nep245::TokenId>,
        amounts: Vec<U128>,
        msg: String,
    ) -> PromiseOrValue<Vec<U128>> {
        require!(
            token_ids.len() == amounts.len() && !amounts.is_empty(),
            "invalid args"
        );
        let _previous_owner_ids = previous_owner_ids;
        let msg = if !msg.is_empty() {
            msg.parse().unwrap_or_panic_display()
        } else {
            DepositMessage::new(sender_id)
        };

        let n = amounts.len();
        self.internal_deposit(
            msg.receiver_id,
            token_ids
                .into_iter()
                .map(|token_id| TokenId::Nep245(PREDECESSOR_ACCOUNT_ID.clone(), token_id))
                .zip(amounts.into_iter().map(|a| a.0)),
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

        PromiseOrValue::Value(vec![U128(0); n])
    }
}
