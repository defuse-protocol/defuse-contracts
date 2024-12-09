use defuse_core::tokens::TokenId;
use defuse_near_utils::{
    UnwrapOrPanic, UnwrapOrPanicError, CURRENT_ACCOUNT_ID, PREDECESSOR_ACCOUNT_ID,
};
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_plugins::{pause, Pausable};
use near_sdk::{json_types::U128, near, require, AccountId, PromiseOrValue};

use crate::{
    contract::{Contract, ContractExt},
    intents::{ext_intents, Intents},
    tokens::DepositMessage,
};

#[near]
impl FungibleTokenReceiver for Contract {
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

        self.deposit(
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
                let _ = ext_intents::ext(CURRENT_ACCOUNT_ID.clone())
                    .execute_intents(msg.execute_intents);
            }
        }

        PromiseOrValue::Value(U128(0))
    }
}
