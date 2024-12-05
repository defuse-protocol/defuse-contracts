use std::{borrow::Cow, collections::HashMap, iter, mem};

use defuse_map_utils::cleanup::DefaultMap;
use defuse_nep245::{MtEvent, MtTransferEvent};
use near_sdk::{json_types::U128, AccountId};

use crate::{
    tokens::{TokenAmounts, TokenId},
    DefuseError, Result,
};

// #[derive(Debug, Default)]
// pub struct Runtime {
//     pub total_supply_deltas: TokenAmounts<HashMap<TokenId, i128>>,
// }

// impl Runtime {
//     #[inline]
//     pub fn finalize(mut self) -> Result<()> {
//         if !mem::take(&mut self).total_supply_deltas.is_empty() {
//             return Err(DefuseError::InvariantViolated);
//         }
//         Ok(())
//     }
// }

type TokenAccountAmounts = HashMap<TokenId, HashMap<AccountId, u128>>;

/// Accumulates deltas and
#[derive(Debug, Default)]
pub struct TransferMatcher {
    deposits: TokenAccountAmounts,
    withdrawals: TokenAccountAmounts,
}

impl TransferMatcher {
    pub fn add_delta(
        &mut self,
        owner_id: AccountId,
        token_id: TokenId,
        delta: i128,
    ) -> Option<i128> {
        let (sub, add) = if delta.is_negative() {
            (&mut self.withdrawals, &mut self.deposits)
        } else {
            // TODO: check order?
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

    pub fn deposit(
        &mut self,
        owner_id: AccountId,
        token_id: TokenId,
        amount: u128,
    ) -> Option<i128> {
        todo!()
        // Self::add(owner_id, token_id, sub, add, amount)
    }

    pub fn withdraw(
        &mut self,
        owner_id: AccountId,
        token_id: TokenId,
        amount: u128,
    ) -> Option<i128> {
        todo!()
    }

    fn add(
        owner_id: AccountId,
        token_id: TokenId,
        sub: &mut TokenAccountAmounts,
        add: &mut TokenAccountAmounts,
        mut amount: u128,
    ) -> Option<u128> {
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

    pub fn finalize(mut self) -> Result<Transfers> {
        let mut transfers = Transfers::default();

        for (token_id, deposits) in self.deposits {
            // get counterpart from withdrawals
            let Some(withdrawals) = self.withdrawals.remove(&token_id) else {
                // no counterpart found
                return Err(DefuseError::InvariantViolated);
            };

            let mut withdrawals = withdrawals.into_iter();
            let mut deposits = deposits.into_iter();

            // get first sender, receiver and their amounts
            let ((mut sender_id, mut sender_amount), (mut receiver_id, mut receiver_amount)) =
                match (withdrawals.next(), deposits.next()) {
                    (Some(w), Some(d)) => (w, d),
                    (None, None) => continue,
                    // no counterpart found
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

                // get min amount and transfer
                let amount = sender_amount.min(receiver_amount);
                transfers
                    .transfer(
                        sender_id.clone(),
                        receiver_id.clone(),
                        token_id.clone(),
                        amount,
                    )
                    .ok_or(DefuseError::BalanceOverflow)?;

                // subtract amount from sender's and receiver's amounts
                sender_amount = sender_amount.saturating_sub(amount);
                receiver_amount = receiver_amount.saturating_sub(amount);
            }

            if sender_amount != 0
                || receiver_amount != 0
                || deposits.len() != 0
                || withdrawals.len() != 0
            {
                // non-zero amount left and was not destributed
                return Err(DefuseError::InvariantViolated);
            }
        }

        if !self.withdrawals.is_empty() {
            // no counterpart found for
            return Err(DefuseError::InvariantViolated);
        }

        Ok(transfers)
    }
}

/// Accumulates transfers between
#[derive(Debug, Default)]
pub struct Transfers(
    /// sender_id -> receiver_id -> token_id -> amount
    HashMap<AccountId, HashMap<AccountId, HashMap<TokenId, u128>>>,
);

impl Transfers {
    pub fn transfer(
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

    pub fn into_event(&self) -> MtEvent<'_> {
        MtEvent::MtTransfer(
            self.0
                .iter()
                .flat_map(|(sender_id, transfers)| iter::repeat(sender_id).zip(transfers))
                .map(|(sender_id, (receiver_id, transfers))| {
                    let (token_ids, amounts) = transfers
                        .into_iter()
                        .map(|(token_id, amount)| (token_id.to_string(), U128(*amount)))
                        .unzip();
                    MtTransferEvent {
                        authorized_id: None,
                        old_owner_id: Cow::Borrowed(&sender_id),
                        new_owner_id: Cow::Borrowed(&receiver_id),
                        token_ids: Cow::Owned(token_ids),
                        amounts: Cow::Owned(amounts),
                        memo: None,
                    }
                })
                .collect::<Vec<_>>()
                .into(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deltas() {
        let mut l = TransferMatcher::default();
        // TODO
        // l.add_delta(owner_id, token_id, delta)
    }
}
