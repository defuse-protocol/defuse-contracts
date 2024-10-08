use defuse_contracts::defuse::{
    intents::tokens::{TokenTransfer, TokenTransferCall, TokenWithdraw},
    DefuseError, Result,
};
use near_sdk::{AccountId, Promise};

use crate::accounts::Account;

use super::{IntentExecutor, State};

impl IntentExecutor<TokenTransfer> for State {
    fn execute_intent(
        &mut self,
        sender_id: &AccountId,
        sender: &mut Account,
        transfer: TokenTransfer,
    ) -> Result<()> {
        if sender_id == &transfer.receiver_id {
            return Err(DefuseError::InvalidSenderReceiver);
        }

        let receiver_deposit = self
            .postponed_deposits
            .entry(transfer.receiver_id)
            .or_default();
        for (token_id, amount) in transfer.token_id_amounts {
            sender.token_balances.withdraw(token_id.clone(), amount)?;
            receiver_deposit.add(token_id, amount)?;
        }

        // TODO: log with memo

        Ok(())
    }
}

impl IntentExecutor<TokenTransferCall> for State {
    fn execute_intent(
        &mut self,
        sender_id: &AccountId,
        sender: &mut Account,
        intent: TokenTransferCall,
    ) -> Result<()> {
        self.internal_transfer_call(sender_id, sender, intent)
            .map(|_promise| ())
    }
}

impl IntentExecutor<TokenWithdraw> for State {
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

impl State {
    #[inline]
    pub fn token_withdraw(
        &mut self,
        sender_id: AccountId,
        sender: &mut Account,
        withdraw: TokenWithdraw,
    ) -> Result<Promise> {
        match withdraw {
            TokenWithdraw::Nep141(withdraw) => self.ft_withdraw(sender_id, sender, withdraw),
            TokenWithdraw::Nep171(withdraw) => self.nft_withdraw(sender_id, sender, withdraw),
            TokenWithdraw::Nep245(withdraw) => self.mt_withdraw(sender_id, sender, withdraw),
        }
    }
}
