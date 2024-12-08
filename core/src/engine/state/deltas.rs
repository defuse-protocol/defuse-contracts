use std::{borrow::Cow, cmp::Reverse, collections::HashMap};

use defuse_crypto::PublicKey;
use defuse_map_utils::cleanup::DefaultMap;
use near_sdk::{AccountId, AccountIdRef};

use crate::{
    engine::Transfers,
    fees::Pips,
    intents::{
        token_diff::TokenDeltas,
        tokens::{FtWithdraw, MtWithdraw, NativeWithdraw, NftWithdraw},
    },
    tokens::{TokenAmounts, TokenId},
    DefuseError, Nonce, Result,
};

use super::{State, StateView};

pub struct Deltas<S> {
    state: S,
    deltas: TransferMatcher,
}

impl<S> Deltas<S> {
    #[inline]
    pub fn new(state: S) -> Self {
        Self {
            state,
            deltas: Default::default(),
        }
    }

    #[inline]
    pub fn finalize(self) -> Result<Transfers> {
        self.deltas
            .finalize()
            .map_err(|unmatched_deltas| DefuseError::InvariantViolated { unmatched_deltas })
    }
}

impl<S> StateView for Deltas<S>
where
    S: StateView,
{
    #[inline]
    fn verifying_contract(&self) -> Cow<'_, AccountIdRef> {
        self.state.verifying_contract()
    }

    #[inline]
    fn wnear_id(&self) -> Cow<'_, AccountIdRef> {
        self.state.wnear_id()
    }

    #[inline]
    fn fee(&self) -> Pips {
        self.state.fee()
    }

    #[inline]
    fn fee_collector(&self) -> Cow<'_, AccountIdRef> {
        self.state.fee_collector()
    }

    #[inline]
    fn has_public_key(&self, account_id: &AccountIdRef, public_key: &PublicKey) -> bool {
        self.state.has_public_key(account_id, public_key)
    }

    #[inline]
    fn iter_public_keys(&self, account_id: &AccountIdRef) -> impl Iterator<Item = PublicKey> + '_ {
        self.state.iter_public_keys(account_id)
    }

    #[inline]
    fn is_nonce_used(&self, account_id: &AccountIdRef, nonce: Nonce) -> bool {
        self.state.is_nonce_used(account_id, nonce)
    }

    #[inline]
    fn balance_of(&self, account_id: &AccountIdRef, token_id: &TokenId) -> u128 {
        self.state.balance_of(account_id, token_id)
    }
}

impl<S> State for Deltas<S>
where
    S: State,
{
    #[must_use]
    #[inline]
    fn add_public_key(&mut self, account_id: AccountId, public_key: PublicKey) -> bool {
        self.state.add_public_key(account_id, public_key)
    }

    #[must_use]
    #[inline]
    fn remove_public_key(&mut self, account_id: AccountId, public_key: PublicKey) -> bool {
        self.state.remove_public_key(account_id, public_key)
    }

    #[must_use]
    #[inline]
    fn commit_nonce(&mut self, account_id: AccountId, nonce: Nonce) -> bool {
        self.state.commit_nonce(account_id, nonce)
    }

    fn internal_deposit(
        &mut self,
        owner_id: AccountId,
        tokens: impl IntoIterator<Item = (TokenId, u128)>,
    ) -> Result<()> {
        for (token_id, amount) in tokens {
            self.state
                .internal_deposit(owner_id.clone(), [(token_id.clone(), amount)])?;
            if !self.deltas.deposit(owner_id.clone(), token_id, amount) {
                return Err(DefuseError::BalanceOverflow);
            }
        }
        Ok(())
    }

    fn internal_withdraw(
        &mut self,
        owner_id: &AccountIdRef,
        tokens: impl IntoIterator<Item = (TokenId, u128)>,
    ) -> Result<()> {
        for (token_id, amount) in tokens {
            self.state
                .internal_withdraw(owner_id, [(token_id.clone(), amount)])?;
            if !self.deltas.withdraw(owner_id.to_owned(), token_id, amount) {
                return Err(DefuseError::BalanceOverflow);
            }
        }
        Ok(())
    }

    #[inline]
    fn ft_withdraw(&mut self, owner_id: &AccountIdRef, withdraw: FtWithdraw) -> Result<()> {
        self.state.ft_withdraw(owner_id, withdraw)
    }

    #[inline]
    fn nft_withdraw(&mut self, owner_id: &AccountIdRef, withdraw: NftWithdraw) -> Result<()> {
        self.state.nft_withdraw(owner_id, withdraw)
    }

    #[inline]
    fn mt_withdraw(&mut self, owner_id: &AccountIdRef, withdraw: MtWithdraw) -> Result<()> {
        self.state.mt_withdraw(owner_id, withdraw)
    }

    #[inline]
    fn native_withdraw(&mut self, owner_id: &AccountIdRef, withdraw: NativeWithdraw) -> Result<()> {
        self.state.native_withdraw(owner_id, withdraw)
    }
}

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

        while let (Some((sender, ref mut send)), Some((receiver, receive))) =
            (withdraw.as_mut(), deposit.as_mut())
        {
            // get min amount and transfer
            let transfer = (*send).min(*receive);
            let _ =
                transfers.transfer(sender.clone(), receiver.clone(), token_id.clone(), transfer);

            // subtract amount from sender's and receiver's amounts
            *send = send.saturating_sub(transfer);
            *receive = receive.saturating_sub(transfer);

            if *send == 0 {
                // select next sender
                withdraw = withdrawals.next();
            }
            if *receive == 0 {
                // select next receiver
                deposit = deposits.next();
            }
        }

        if let Some((_, send)) = withdraw {
            return Err(withdrawals
                .try_fold(send, |total, (_, s)| total.checked_add(s))
                .and_then(|total| i128::try_from(total).ok())
                .and_then(i128::checked_neg));
        }
        if let Some((_, receive)) = deposit {
            return Err(deposits
                .try_fold(receive, |total, (_, r)| total.checked_add(r))
                .and_then(|total| i128::try_from(total).ok()));
        }

        Ok(())
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
