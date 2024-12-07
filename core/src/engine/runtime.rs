use std::{borrow::Cow, collections::HashMap, iter};

use defuse_map_utils::cleanup::DefaultMap;
use defuse_nep245::{MtEvent, MtTransferEvent};
use near_sdk::{json_types::U128, AccountId};

use crate::{
    tokens::{TokenAmounts, TokenId},
    Result,
};

// TODO: docs
/// Accumulates deltas and
#[derive(Debug, Default)]
pub struct DeltaMatcher(HashMap<TokenId, TokenAmounts<HashMap<AccountId, i128>>>);

impl DeltaMatcher {
    #[inline]
    pub fn add_delta(
        &mut self,
        owner_id: AccountId,
        token_id: TokenId,
        delta: i128,
    ) -> Option<i128> {
        self.0.entry_or_default(token_id).add_delta(owner_id, delta)
    }

    pub fn finalize(self) -> Result<Transfers> {
        let mut transfers = Transfers::default();
        // TODO
        for (token_id, token_transfers) in self.0 {
            // TODO
            // token_transfers.finalize_into(token_id, &mut transfers)?;
        }
        Ok(transfers)
    }
}

// #[derive(Debug, Default)]
// pub struct TokenTransfers {
//     withdrawals: HashMap<AccountId, u128>,
//     deposits: HashMap<AccountId, u128>,
// }

// impl TokenTransfers {
//     #[inline]
//     pub fn deposit(&mut self, owner_id: AccountId, amount: u128) -> Result<()> {
//         Self::sub_add(&mut self.withdrawals, &mut self.deposits, owner_id, amount)
//     }

//     #[inline]
//     pub fn withdraw(&mut self, owner_id: AccountId, amount: u128) -> Result<()> {
//         Self::sub_add(&mut self.deposits, &mut self.withdrawals, owner_id, amount)
//     }

//     #[inline]
//     pub fn add_delta(&mut self, owner_id: AccountId, delta: i128) -> Result<()> {
//         let amount = delta.unsigned_abs();
//         if delta.is_negative() {
//             self.withdraw(owner_id, amount)
//         } else {
//             self.deposit(owner_id, amount)
//         }
//     }

//     fn sub_add(
//         sub: &mut HashMap<AccountId, u128>,
//         add: &mut HashMap<AccountId, u128>,
//         owner_id: AccountId,
//         mut amount: u128,
//     ) -> Result<()> {
//         if let Some(s) = sub.get_mut(&owner_id) {
//             let a = (*s).min(amount);
//             amount = amount.saturating_sub(a);
//             *s = s.saturating_sub(a);
//             if *s == 0 {
//                 sub.remove(k)
//             }
//         }
//         // let mut sub = sub.entry_or_default(owner_id);
//         // let s = sub.min(amount);
//         // *sub = sub.saturating_sub(s);
//         // amount = amount.saturating_sub(s);

//         if amount > 0 {
//             let mut add = add.entry_or_default(sub.key().clone());
//             *add = add
//                 .checked_add(amount)
//                 .ok_or(DefuseError::BalanceOverflow)?;
//         }

//         Ok(())
//     }

//     pub fn finalize_into(self, token_id: TokenId, transfers: &mut Transfers) -> Result<()> {
//         let [mut withdrawals, mut deposits] =
//             [self.withdrawals, self.deposits].map(IntoIterator::into_iter);

//         // get first sender, receiver and their amounts
//         let ((mut sender_id, mut sender_amount), (mut receiver_id, mut receiver_amount)) =
//             match (withdrawals.next(), deposits.next()) {
//                 (Some(w), Some(d)) => (w, d),
//                 // nothing to match
//                 (None, None) => return Ok(()),
//                 // no counterpart found
//                 _ => return Err(DefuseError::InvariantViolated),
//             };

//         loop {
//             // find next sender with non-zero amount
//             if sender_amount == 0 {
//                 let Some((s, a)) = withdrawals.next() else {
//                     break;
//                 };
//                 (sender_id, sender_amount) = (s, a);
//                 continue;
//             }

//             // find next receiver with non-zero amount
//             if receiver_amount == 0 {
//                 let Some((r, a)) = deposits.next() else {
//                     break;
//                 };
//                 (receiver_id, receiver_amount) = (r, a);
//                 continue;
//             }

//             // get min amount and transfer
//             let amount = sender_amount.min(receiver_amount);
//             transfers
//                 .transfer(
//                     sender_id.clone(),
//                     receiver_id.clone(),
//                     token_id.clone(),
//                     amount,
//                 )
//                 .ok_or(DefuseError::BalanceOverflow)?;

//             // subtract amount from sender's and receiver's amounts
//             sender_amount = sender_amount.saturating_sub(amount);
//             receiver_amount = receiver_amount.saturating_sub(amount);
//         }

//         if sender_amount != 0
//             || receiver_amount != 0
//             || deposits.len() != 0
//             || withdrawals.len() != 0
//         {
//             // non-zero amount left and was not destributed
//             return Err(DefuseError::InvariantViolated);
//         }

//         Ok(())
//     }
// }

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

    pub fn into_event(&self) -> Option<MtEvent<'_>> {
        if self.0.is_empty() {
            return None;
        }
        Some(MtEvent::MtTransfer(
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
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deltas() {
        let mut l = DeltaMatcher::default();
        // TODO
        // l.add_delta(owner_id, token_id, delta)
    }
}
