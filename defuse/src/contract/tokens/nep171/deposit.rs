use defuse_core::tokens::TokenId;
use defuse_near_utils::{
    UnwrapOrPanic, UnwrapOrPanicError, CURRENT_ACCOUNT_ID, PREDECESSOR_ACCOUNT_ID,
};
use near_contract_standards::non_fungible_token::core::NonFungibleTokenReceiver;
use near_plugins::{pause, Pausable};
use near_sdk::{near, AccountId, PromiseOrValue};

use crate::{
    contract::{Contract, ContractExt},
    intents::{ext_intents, Intents},
    tokens::DepositMessage,
};

#[near]
impl NonFungibleTokenReceiver for Contract {
    /// Deposit non-fungible token.
    ///
    /// `msg` contains [`AccountId`] of the internal recipient.
    /// Empty `msg` means deposit to `sender_id`
    #[pause]
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: near_contract_standards::non_fungible_token::TokenId,
        msg: String,
    ) -> PromiseOrValue<bool> {
        let _previous_owner_id = previous_owner_id;
        let msg = if !msg.is_empty() {
            msg.parse().unwrap_or_panic_display()
        } else {
            DepositMessage::new(sender_id)
        };

        self.internal_deposit(
            msg.receiver_id,
            [(TokenId::Nep171(PREDECESSOR_ACCOUNT_ID.clone(), token_id), 1)],
            Some("deposit"),
        )
        .unwrap_or_panic();

        if !msg.execute_intents.is_empty() {
            if msg.refund_if_fails {
                self.execute_intents(msg.execute_intents);
            } else {
                // detach promise
                ext_intents::ext(CURRENT_ACCOUNT_ID.clone()).execute_intents(msg.execute_intents);
            }
        }

        PromiseOrValue::Value(false)
    }
}
