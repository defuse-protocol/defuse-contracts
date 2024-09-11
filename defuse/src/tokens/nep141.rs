use defuse_contracts::defuse::{
    token::{DepositMessage, TokenId},
    DefuseError,
};
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::{env, json_types::U128, near, serde_json, AccountId, PromiseOrValue};

use crate::{DefuseImpl, DefuseImplExt};

#[near]
impl FungibleTokenReceiver for DefuseImpl {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        self.internal_ft_on_transfer(sender_id, amount, &msg)
            .map(|()| PromiseOrValue::Value(U128(0)))
            .unwrap()
    }
}

impl DefuseImpl {
    fn internal_ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: &str,
    ) -> Result<(), DefuseError> {
        // TODO: empty msg
        let msg: DepositMessage = Some(msg)
            .filter(|msg| !msg.is_empty())
            .map(serde_json::from_str)
            .transpose()?
            .unwrap_or_default();

        let token = TokenId::Nep141(env::predecessor_account_id());
        let account = self
            .accounts
            .get_or_insert(msg.deposit_to.unwrap_or(sender_id));
        account.token_balances.deposit(token, amount.0)?;

        for action in msg.actions {
            // TODO: pass predecessor_id?
            self.execute_action(action)?;
        }

        Ok(())
    }
}
