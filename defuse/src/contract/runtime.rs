use core::{iter, mem};
use std::{borrow::Cow, collections::HashMap};

use defuse_core::{tokens::TokenId, DefuseError, Result};
use defuse_map_utils::cleanup::DefaultMap;
use defuse_near_utils::UnwrapOrPanic;
use defuse_nep245::{MtEvent, MtEventEmit, MtTransferEvent};
use near_sdk::{json_types::U128, AccountId};

type TokenAccountAmounts = HashMap<TokenId, HashMap<AccountId, u128>>;

#[derive(Debug, Default)]
pub struct TransferLogger {
    deposits: TokenAccountAmounts,
    withdrawals: TokenAccountAmounts,
}

impl TransferLogger {
    pub fn add_delta(
        &mut self,
        owner_id: AccountId,
        token_id: TokenId,
        delta: i128,
    ) -> Option<i128> {
        let (sub, add) = if delta.is_negative() {
            (&mut self.withdrawals, &mut self.deposits)
        } else {
            (&mut self.deposits, &mut self.withdrawals)
        };

        let mut amount = delta.unsigned_abs();

        let mut token_sub = sub.entry_or_default(token_id.clone());
        let mut sub = token_sub.entry_or_default(owner_id.clone());

        // sub as much as we can
        let old_sub = *sub;
        *sub = sub.saturating_sub(amount);
        amount = amount.saturating_sub(old_sub);

        if amount == 0 {
            // TODO: early return
        }

        let mut token_add = add.entry_or_default(token_id);
        let mut add = token_add.entry_or_default(owner_id);
        *add = add.checked_add(amount)?;

        // TODO
        Some(0)
    }

    pub fn finalize(mut self) -> Result<()> {
        let mut transfers = Transfers::default();

        for (token_id, deposits) in self.deposits {
            let Some(withdrawals) = self.withdrawals.remove(&token_id) else {
                return Err(DefuseError::InvariantViolated);
            };

            let mut withdrawals = withdrawals.into_iter();
            let mut deposits = deposits.into_iter();

            let ((mut sender_id, mut sender_amount), (mut receiver_id, mut receiver_amount)) =
                match (withdrawals.next(), deposits.next()) {
                    (Some(w), Some(d)) => (w, d),
                    (None, None) => continue,
                    _ => return Err(DefuseError::InvariantViolated),
                };

            loop {
                // find next sender with non-zero amount
                if sender_amount == 0 {
                    let Some((s, a)) = withdrawals.next() else {
                        break;
                    };
                    (sender_id, sender_amount) = (s, a);
                    continue;
                }

                // find next receiver with non-zero amount
                if receiver_amount == 0 {
                    let Some((r, a)) = deposits.next() else {
                        break;
                    };
                    (receiver_id, receiver_amount) = (r, a);
                    continue;
                }

                let send = sender_amount.min(receiver_amount);
                transfers
                    .transfer(
                        sender_id.clone(),
                        receiver_id.clone(),
                        token_id.clone(),
                        send,
                    )
                    .ok_or(DefuseError::BalanceOverflow)?;

                sender_amount = sender_amount.saturating_sub(send);
                receiver_amount = receiver_amount.saturating_sub(send);
            }

            if sender_amount != 0
                || receiver_amount != 0
                || deposits.len() != 0
                || withdrawals.len() != 0
            {
                return Err(DefuseError::InvariantViolated);
            }
        }

        if !self.withdrawals.is_empty() {
            return Err(DefuseError::InvariantViolated);
        }

        transfers.emit();

        Ok(())
    }
}

#[derive(Debug, Default)]
struct Transfers(HashMap<AccountId, HashMap<AccountId, HashMap<TokenId, u128>>>);

impl Transfers {
    fn transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        amount: u128,
    ) -> Option<u128> {
        let mut sender = self.0.entry_or_default(sender_id);
        let mut receiver = sender.entry_or_default(receiver_id);
        let mut transfer = receiver.entry_or_default(token_id);
        *transfer = transfer.checked_add(amount)?;
        Some(*transfer)
    }

    fn emit(self) {
        MtEvent::MtTransfer(
            self.0
                .into_iter()
                .flat_map(|(sender_id, transfers)| iter::repeat(sender_id).zip(transfers))
                .map(|(sender_id, (receiver_id, transfers))| {
                    let (token_ids, amounts) = transfers
                        .into_iter()
                        .map(|(token_id, amount)| (token_id.to_string(), U128(amount)))
                        .unzip();
                    MtTransferEvent {
                        authorized_id: None,
                        old_owner_id: Cow::Owned(sender_id),
                        new_owner_id: Cow::Owned(receiver_id),
                        token_ids: Cow::Owned(token_ids),
                        amounts: Cow::Owned(amounts),
                        memo: None,
                    }
                })
                .collect::<Vec<_>>()
                .into(),
        )
        .emit();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deltas() {
        let mut l = TransferLogger::default();
        // TODO
        // l.add_delta(owner_id, token_id, delta)
    }
}
