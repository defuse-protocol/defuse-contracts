use defuse_contracts::defuse::{
    intents::tokens::{FtWithdraw, MtBatchTransfer, MtBatchTransferCall, MtWithdraw, NftWithdraw},
    DefuseError, Result,
};
use near_sdk::{require, AccountId};

use crate::accounts::Account;

use super::{IntentExecutor, State};

impl IntentExecutor<MtBatchTransfer> for State {
    fn execute_intent(
        &mut self,
        sender_id: &AccountId,
        sender: &mut Account,
        MtBatchTransfer {
            receiver_id,
            token_ids,
            amounts,
            memo: _memo,
        }: MtBatchTransfer,
    ) -> Result<()> {
        require!(sender_id != &receiver_id, "sender_id == receiver_id");
        require!(
            token_ids.len() == amounts.len(),
            "token_ids.len() != amounts.len()"
        );

        let receiver_deposit = self.postponed_deposits.entry(receiver_id).or_default();

        for (token_id, amount) in token_ids.into_iter().zip(amounts.into_iter().map(|a| a.0)) {
            if amount == 0 {
                return Err(DefuseError::ZeroAmount);
            }
            sender.token_balances.withdraw(token_id.clone(), amount)?;
            receiver_deposit.add(token_id, amount)?;
        }

        // TODO: log with memo

        Ok(())
    }
}

impl IntentExecutor<MtBatchTransferCall> for State {
    #[inline]
    fn execute_intent(
        &mut self,
        sender_id: &AccountId,
        sender: &mut Account,
        intent: MtBatchTransferCall,
    ) -> Result<()> {
        self.internal_mt_batch_transfer_call(sender_id, sender, intent)
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
