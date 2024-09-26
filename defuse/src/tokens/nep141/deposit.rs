use defuse_contracts::{defuse::tokens::TokenId, utils::cache::PREDECESSOR_ACCOUNT_ID};
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
        let deposit_to = if !msg.is_empty() {
            msg.parse().unwrap()
        } else {
            sender_id
        };

        let token_id = TokenId::Nep141(PREDECESSOR_ACCOUNT_ID.clone());
        self.total_supplies
            .deposit(token_id.clone(), amount.0)
            .unwrap();
        self.accounts
            .get_or_create(deposit_to)
            .token_balances
            .deposit(token_id, amount.0)
            .unwrap();

        PromiseOrValue::Value(U128(0))
    }
}
