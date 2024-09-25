use defuse_contracts::{defuse::tokens::TokenId, utils::cache::PREDECESSOR_ACCOUNT_ID};
use near_contract_standards::non_fungible_token::core::NonFungibleTokenReceiver;
use near_sdk::{near, AccountId, PromiseOrValue};

use crate::{DefuseImpl, DefuseImplExt};

#[near]
impl NonFungibleTokenReceiver for DefuseImpl {
    /// Deposit non-fungible token.
    ///
    /// `msg` contains [`AccountId`] of the internal recipient.
    /// Empty `msg` means deposit to `sender_id`
    #[allow(unused_variables)]
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: near_contract_standards::non_fungible_token::TokenId,
        msg: String,
    ) -> PromiseOrValue<bool> {
        let deposit_to = if !msg.is_empty() {
            msg.parse().unwrap()
        } else {
            sender_id
        };

        self.accounts
            .get_or_create(deposit_to)
            .token_balances
            .deposit(TokenId::Nep171(PREDECESSOR_ACCOUNT_ID.clone(), token_id), 1)
            .unwrap();

        PromiseOrValue::Value(false)
    }
}
