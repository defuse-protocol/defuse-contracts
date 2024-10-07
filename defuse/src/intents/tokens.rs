use defuse_contracts::defuse::{
    intents::tokens::{TokenTransfer, TokenWithdraw},
    Result,
};
use near_sdk::AccountId;

use crate::accounts::Account;

use super::runtime::{IntentExecutor, Runtime};

impl<'a> IntentExecutor<TokenTransfer> for Runtime<'a> {
    fn execute_intent(
        &mut self,
        _sender_id: &AccountId,
        sender: &mut Account,
        intent: TokenTransfer,
    ) -> Result<()> {
        let receiver_deposit = self
            .postponed_deposits
            .entry(intent.recipient_id)
            .or_default();
        for (token_id, amount) in intent.tokens {
            sender.token_balances.withdraw(token_id.clone(), amount)?;
            receiver_deposit.add(token_id, amount)?;
        }
        Ok(())
    }
}

impl<'a> IntentExecutor<TokenWithdraw> for Runtime<'a> {
    fn execute_intent(
        &mut self,
        account_id: &AccountId,
        account: &mut Account,
        intent: TokenWithdraw,
    ) -> Result<()> {
        self.token_withdraw(account_id.clone(), account, intent)
            // detach promise
            .map(|_promise| ())
    }
}
