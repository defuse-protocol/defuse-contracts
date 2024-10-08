use defuse_contracts::defuse::{
    intents::tokens::{FtWithdraw, MtWithdraw, NftWithdraw, TokenTransfer, TokenTransferCall},
    DefuseError, Result,
};
use near_sdk::AccountId;

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
    #[inline]
    fn execute_intent(
        &mut self,
        sender_id: &AccountId,
        sender: &mut Account,
        intent: TokenTransferCall,
    ) -> Result<()> {
        self.internal_transfer_call(sender_id, sender, intent)
            // detach
            .map(|_promise| ())
    }
}

impl IntentExecutor<FtWithdraw> for State {
    #[inline]
    fn execute_intent(
        &mut self,
        sender_id: &AccountId,
        sender: &mut Account,
        intent: FtWithdraw,
    ) -> Result<()> {
        self.ft_withdraw(sender_id.clone(), sender, intent)
            // detach
            .map(|_promise| ())
    }
}

impl IntentExecutor<NftWithdraw> for State {
    #[inline]
    fn execute_intent(
        &mut self,
        sender_id: &AccountId,
        sender: &mut Account,
        intent: NftWithdraw,
    ) -> Result<()> {
        self.nft_withdraw(sender_id.clone(), sender, intent)
            // detach
            .map(|_promise| ())
    }
}

impl IntentExecutor<MtWithdraw> for State {
    #[inline]
    fn execute_intent(
        &mut self,
        sender_id: &AccountId,
        sender: &mut Account,
        intent: MtWithdraw,
    ) -> Result<()> {
        self.mt_withdraw(sender_id.clone(), sender, intent)
            // detach
            .map(|_promise| ())
    }
}
