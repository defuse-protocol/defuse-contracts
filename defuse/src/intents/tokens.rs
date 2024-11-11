use defuse_contracts::{
    defuse::{
        intents::tokens::{
            FtWithdraw, MtBatchTransfer, MtBatchTransferCall, MtWithdraw, NativeWithdraw,
            NftWithdraw,
        },
        Result,
    },
    nep245::{MtEventEmit, MtTransferEvent},
};
use near_sdk::{require, AccountId, Gas};

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
            memo,
        }: MtBatchTransfer,
    ) -> Result<()> {
        require!(
            sender_id != &receiver_id && token_ids.len() == amounts.len() && !amounts.is_empty(),
            "invalid args"
        );

        let mut event = MtTransferEvent {
            authorized_id: None,
            old_owner_id: sender_id.into(),
            new_owner_id: receiver_id.clone().into(),
            token_ids: Vec::with_capacity(token_ids.len()).into(),
            amounts: amounts.clone().into(),
            memo: memo.map(Into::into),
        };

        let receiver_deposit = self.postponed_deposits.entry(receiver_id).or_default();

        for (token_id, amount) in token_ids.into_iter().zip(amounts.into_iter().map(|a| a.0)) {
            require!(amount > 0, "zero amount");
            event.token_ids.to_mut().push(token_id.to_string());

            sender.token_balances.withdraw(token_id.clone(), amount)?;
            receiver_deposit.add(token_id, amount)?;
        }

        [event].emit();

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
        const GAS_FOR_MT_ON_TRANSFER: Gas = Gas::from_tgas(15);

        self.internal_mt_batch_transfer_call(sender_id, sender, intent, GAS_FOR_MT_ON_TRANSFER)
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

impl IntentExecutor<NativeWithdraw> for State {
    fn execute_intent(
        &mut self,
        account_id: &AccountId,
        account: &mut Account,
        intent: NativeWithdraw,
    ) -> Result<()> {
        self.native_withdraw(account_id, account, intent)
            // detach
            .map(|_promise| ())
    }
}
