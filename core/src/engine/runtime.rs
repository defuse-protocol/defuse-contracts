use std::{borrow::Cow, cmp::Reverse, collections::HashMap, iter};

use defuse_map_utils::cleanup::DefaultMap;
use defuse_nep245::{MtEvent, MtTransferEvent};
use near_sdk::{json_types::U128, AccountId};

use crate::{
    intents::token_diff::TokenDeltas,
    tokens::{TokenAmounts, TokenId},
};

// TODO: docs
/// Accumulates deltas and
#[derive(Debug, Default)]
pub struct TransferMatcher(HashMap<TokenId, TokenTransferMatcher>);

impl TransferMatcher {
    #[inline]
    pub fn deposit(&mut self, owner_id: AccountId, token_id: TokenId, amount: u128) -> bool {
        self.0.entry_or_default(token_id).deposit(owner_id, amount)
    }

    #[inline]
    pub fn withdraw(&mut self, owner_id: AccountId, token_id: TokenId, amount: u128) -> bool {
        self.0.entry_or_default(token_id).withdraw(owner_id, amount)
    }
    #[inline]
    pub fn add_delta(&mut self, owner_id: AccountId, token_id: TokenId, delta: i128) -> bool {
        self.0.entry_or_default(token_id).add_delta(owner_id, delta)
    }

    pub fn finalize(self) -> Result<Transfers, Option<TokenDeltas>> {
        let mut transfers = Transfers::default();
        let mut unmatched_deltas = TokenDeltas::default();
        for (token_id, deltas) in self.0 {
            if let Err(unmatched) = deltas.finalize_into(token_id.clone(), &mut transfers) {
                if unmatched
                    .and_then(|d| unmatched_deltas.add_delta(token_id, d))
                    .is_none()
                {
                    return Err(None);
                }
            }
        }
        if !unmatched_deltas.is_empty() {
            return Err(Some(unmatched_deltas));
        }
        Ok(transfers)
    }
}

type AccountAmounts = TokenAmounts<HashMap<AccountId, u128>>;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct TokenTransferMatcher {
    deposits: AccountAmounts,
    withdrawals: AccountAmounts,
}

impl TokenTransferMatcher {
    #[inline]
    pub fn deposit(&mut self, owner_id: AccountId, amount: u128) -> bool {
        Self::sub_add(&mut self.withdrawals, &mut self.deposits, owner_id, amount)
    }

    #[inline]
    pub fn withdraw(&mut self, owner_id: AccountId, amount: u128) -> bool {
        Self::sub_add(&mut self.deposits, &mut self.withdrawals, owner_id, amount)
    }

    #[inline]
    pub fn add_delta(&mut self, owner_id: AccountId, delta: i128) -> bool {
        let amount = delta.unsigned_abs();
        if delta.is_negative() {
            self.withdraw(owner_id, amount)
        } else {
            self.deposit(owner_id, amount)
        }
    }

    fn sub_add(
        sub: &mut AccountAmounts,
        add: &mut AccountAmounts,
        owner_id: AccountId,
        mut amount: u128,
    ) -> bool {
        let s = sub.balance_of(&owner_id);
        if s > 0 {
            let a = s.min(amount);
            sub.withdraw(owner_id.clone(), a);
            amount = amount.saturating_sub(a);
            if amount == 0 {
                return true;
            }
        }
        add.deposit(owner_id, amount).is_some()
    }

    pub fn finalize_into(
        self,
        token_id: TokenId,
        transfers: &mut Transfers,
    ) -> Result<(), Option<i128>> {
        let [mut deposits, mut withdrawals] = [self.deposits, self.withdrawals].map(|amounts| {
            let mut amounts: Vec<_> = amounts.into_iter().collect();
            amounts.sort_unstable_by_key(|(_, amount)| Reverse(*amount));
            amounts.into_iter()
        });

        let (mut deposit, mut withdraw) = (deposits.next(), withdrawals.next());

        loop {
            match (withdraw.as_mut(), deposit.as_mut()) {
                (Some((sender, ref mut send)), Some((receiver, receive))) => {
                    // get min amount and transfer
                    let transfer = (*send).min(*receive);
                    let _ = transfers.transfer(
                        sender.clone(),
                        receiver.clone(),
                        token_id.clone(),
                        transfer,
                    );

                    // subtract amount from sender's and receiver's amounts
                    *send = send.saturating_sub(transfer);
                    *receive = receive.saturating_sub(transfer);

                    if *send == 0 {
                        withdraw = withdrawals.next();
                    }
                    if *receive == 0 {
                        deposit = deposits.next();
                    }
                }
                (None, None) => return Ok(()),
                (Some((_, send)), None) => {
                    return Err(withdrawals
                        .try_fold(*send, |total, (_, s)| total.checked_add(s))
                        .and_then(|total| i128::try_from(total).ok())
                        .and_then(i128::checked_neg));
                }
                (None, Some((_, receive))) => {
                    return Err(deposits
                        .try_fold(*receive, |total, (_, r)| total.checked_add(r))
                        .and_then(|total| i128::try_from(total).ok()))
                }
            }
        }
    }
}

/// Accumulates transfers between
#[derive(Debug, Default)]
pub struct Transfers(
    /// sender_id -> receiver_id -> token_id -> amount
    HashMap<AccountId, HashMap<AccountId, HashMap<TokenId, u128>>>,
);

impl Transfers {
    #[must_use]
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

    pub fn as_event(&self) -> Option<MtEvent<'_>> {
        if self.0.is_empty() {
            return None;
        }
        Some(MtEvent::MtTransfer(
            self.0
                .iter()
                .flat_map(|(sender_id, transfers)| iter::repeat(sender_id).zip(transfers))
                .map(|(sender_id, (receiver_id, transfers))| {
                    let (token_ids, amounts) = transfers
                        .iter()
                        .map(|(token_id, amount)| (token_id.to_string(), U128(*amount)))
                        .unzip();
                    MtTransferEvent {
                        authorized_id: None,
                        old_owner_id: Cow::Borrowed(sender_id),
                        new_owner_id: Cow::Borrowed(receiver_id),
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
        let mut deltas = TransferMatcher::default();
        let [a, b, c, d, e, f, g, h, i]: [AccountId; 9] =
            ["a", "b", "c", "d", "e", "f", "g", "h", "i"]
                .map(|s| format!("{s}.near").parse().unwrap());
        let [ft1, ft2] =
            ["ft1", "ft2"].map(|a| TokenId::Nep141(format!("{a}.near").parse().unwrap()));

        for (owner, token_id, delta) in [
            (a, ft1.clone(), -5),
            (b, ft1.clone(), 4),
            (c, ft1.clone(), 3),
            (d, ft1.clone(), -10),
            (e, ft1.clone(), -1),
            (f, ft1.clone(), 10),
            (g, ft1.clone(), -1),
            (h, ft2.clone(), -1),
            (i, ft2.clone(), 1),
        ] {
            assert!(deltas.add_delta(owner, token_id, delta));
        }

        deltas.finalize().unwrap();
        // TODO
    }
}
