use defuse_contracts::{
    defuse::tokens::TokenId, nep245::receiver::MultiTokenReceiver,
    utils::cache::PREDECESSOR_ACCOUNT_ID,
};
use near_sdk::{json_types::U128, near, require, AccountId, PromiseOrValue};

use crate::{DefuseImpl, DefuseImplExt};

#[near]
impl MultiTokenReceiver for DefuseImpl {
    /// Deposit multi-tokens.
    ///
    /// `msg` contains [`AccountId`] of the internal recipient.
    /// Empty `msg` means deposit to `sender_id`
    #[allow(unused_variables)]
    fn mt_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_ids: Vec<AccountId>,
        token_ids: Vec<defuse_contracts::nep245::TokenId>,
        amounts: Vec<U128>,
        msg: String,
    ) -> near_sdk::PromiseOrValue<Vec<U128>> {
        require!(
            token_ids.len() == amounts.len(),
            "token_ids should be the same length as amounts"
        );

        let deposit_to = if !msg.is_empty() {
            msg.parse().unwrap()
        } else {
            sender_id
        };

        let receiver_tokens = self.accounts.get_or_create(deposit_to);
        for (token_id, amount) in token_ids.into_iter().zip(&amounts) {
            receiver_tokens
                .token_balances
                .deposit(
                    TokenId::Nep245(PREDECESSOR_ACCOUNT_ID.clone(), token_id.parse().unwrap()),
                    amount.0,
                )
                .unwrap();
        }

        PromiseOrValue::Value(amounts.into_iter().map(|_| U128(0)).collect())
    }
}
