use defuse_contracts::defuse::{
    tokens::{DepositMessage, TokenId},
    DefuseError,
};
use near_contract_standards::non_fungible_token::core::NonFungibleTokenReceiver;
use near_sdk::{env, near, serde_json, AccountId, PromiseOrValue};

use crate::{DefuseImpl, DefuseImplExt};

#[near]
impl NonFungibleTokenReceiver for DefuseImpl {
    #[allow(unused_variables)]
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: near_contract_standards::non_fungible_token::TokenId,
        msg: String,
    ) -> PromiseOrValue<bool> {
        self.internal_nft_on_transfer(sender_id, token_id, &msg)
            .map(|()| PromiseOrValue::Value(false))
            .unwrap()
    }
}

impl DefuseImpl {
    fn internal_nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        token_id: near_contract_standards::non_fungible_token::TokenId,
        msg: &str,
    ) -> Result<(), DefuseError> {
        let msg: DepositMessage = Some(msg)
            .filter(|msg| !msg.is_empty())
            .map(serde_json::from_str)
            .transpose()?
            .unwrap_or_default();

        let token = TokenId::Nep171(env::predecessor_account_id(), token_id);
        let account = self
            .accounts
            .get_or_create(msg.deposit_to.unwrap_or(sender_id));
        account.token_balances.deposit(token, 1)?;

        for action in msg.actions {
            // TODO: pass predecessor_id?
            self.execute_action(action)?;
        }

        Ok(())
    }
}
