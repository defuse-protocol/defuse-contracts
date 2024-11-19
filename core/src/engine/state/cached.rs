use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

use defuse_bitmap::{U248, U256};
use defuse_crypto::PublicKey;
use near_sdk::{AccountId, AccountIdRef};

use crate::{
    fees::Pips,
    intents::{
        token_diff::TokenDeltas,
        tokens::{FtWithdraw, MtBatchTransfer, MtWithdraw, NftWithdraw},
    },
    tokens::{TokenAmounts, TokenId},
    DefuseError, Nonce, Nonces, Result,
};

use super::{State, StateView};

#[derive(Debug)]
pub struct CachedState<W: StateView> {
    view: W,
    accounts: CachedAccounts,
    // TODO: finalize
    total_supply_deltas: TokenDeltas,
}

impl<W> CachedState<W>
where
    W: StateView,
{
    #[inline]
    pub fn new(view: W) -> Self {
        Self {
            view,
            accounts: Default::default(),
            total_supply_deltas: Default::default(),
        }
    }

    fn internal_deposit(
        &mut self,
        owner_id: AccountId,
        token_id: TokenId,
        amount: u128,
    ) -> Option<u128> {
        let account = self.accounts.get_or_create(owner_id.clone());
        if account.token_amounts.get(&token_id).is_none() {
            account
                .token_amounts
                .deposit(token_id.clone(), self.view.balance_of(&owner_id, &token_id))?;
        }
        account.token_amounts.deposit(token_id, amount)
    }

    fn internal_withdraw(
        &mut self,
        owner_id: AccountId,
        token_id: TokenId,
        amount: u128,
    ) -> Option<u128> {
        let account = self.accounts.get_or_create(owner_id.clone());
        if account.token_amounts.get(&token_id).is_none() {
            account
                .token_amounts
                .deposit(token_id.clone(), self.view.balance_of(&owner_id, &token_id))?;
        }
        account.token_amounts.withdraw(token_id, amount)
    }

    #[inline]
    pub fn finalize(self) -> Result<(), TokenDeltas> {
        self.total_supply_deltas
            .is_empty()
            .then_some(())
            .ok_or(self.total_supply_deltas)
    }
}

impl<W> StateView for CachedState<W>
where
    W: StateView,
{
    #[inline]
    fn verifying_contract(&self) -> Cow<'_, AccountIdRef> {
        self.view.verifying_contract()
    }

    #[inline]
    fn wnear_id(&self) -> Cow<'_, AccountIdRef> {
        self.view.wnear_id()
    }

    #[inline]
    fn fee(&self) -> Pips {
        self.view.fee()
    }

    #[inline]
    fn fee_collector(&self) -> Cow<'_, AccountIdRef> {
        self.view.fee_collector()
    }

    fn has_public_key(&self, account_id: &AccountIdRef, public_key: &PublicKey) -> bool {
        if let Some(account) = self.accounts.get(account_id) {
            if account.public_keys_added.contains(public_key) {
                return true;
            }
            if account.public_keys_removed.contains(public_key) {
                return false;
            }
        }
        self.view.has_public_key(account_id, public_key)
    }

    fn iter_public_keys(&self, account_id: &AccountIdRef) -> impl Iterator<Item = PublicKey> + '_ {
        let account = self.accounts.get(account_id);
        self.view
            .iter_public_keys(account_id)
            .filter(move |pk| account.is_none_or(|a| !a.public_keys_removed.contains(pk)))
            .chain(
                account
                    .map(|a| &a.public_keys_added)
                    .into_iter()
                    .flatten()
                    .copied(),
            )
    }

    fn is_nonce_used(&self, account_id: &AccountIdRef, nonce: Nonce) -> bool {
        self.accounts
            .get(account_id)
            .map(|account| account.is_nonce_used(nonce))
            .unwrap_or_default()
            || self.view.is_nonce_used(account_id, nonce)
    }

    fn balance_of(&self, account_id: &AccountIdRef, token_id: &TokenId) -> u128 {
        self.accounts
            .get(account_id)
            .and_then(|account| account.token_amounts.get(token_id).copied())
            .unwrap_or_else(|| self.view.balance_of(account_id, token_id))
    }
}

impl<W> State for CachedState<W>
where
    W: StateView,
{
    #[must_use]
    fn add_public_key(&mut self, account_id: AccountId, public_key: PublicKey) -> bool {
        let had = self.has_public_key(&account_id, &public_key);
        let account = self.accounts.get_or_create(account_id.clone());
        if had {
            account.public_keys_removed.remove(&public_key)
        } else {
            account.public_keys_added.insert(public_key)
        }
    }

    #[must_use]
    fn remove_public_key(&mut self, account_id: AccountId, public_key: PublicKey) -> bool {
        let had = self.has_public_key(&account_id, &public_key);
        let account = self.accounts.get_or_create(account_id.clone());
        if had {
            account.public_keys_removed.insert(public_key)
        } else {
            account.public_keys_added.remove(&public_key)
        }
    }

    #[must_use]
    fn commit_nonce(&mut self, account_id: AccountId, nonce: Nonce) -> bool {
        if self.is_nonce_used(&account_id, nonce) {
            return false;
        }
        self.accounts.get_or_create(account_id).commit_nonce(nonce)
    }

    fn internal_add_delta(
        &mut self,
        owner_id: AccountId,
        token_id: TokenId,
        delta: i128,
    ) -> Option<u128> {
        self.total_supply_deltas
            .add_delta(token_id.clone(), delta)?;

        let amount = delta.unsigned_abs();
        if delta.is_negative() {
            self.internal_withdraw(owner_id, token_id, amount)
        } else {
            self.internal_deposit(owner_id, token_id, amount)
        }
    }

    fn mt_transfer(&mut self, sender_id: AccountId, transfer: MtBatchTransfer) -> Result<()> {
        if transfer.receiver_id == sender_id
            || transfer.amounts.is_empty()
            || transfer.token_ids.len() != transfer.amounts.len()
        {
            return Err(DefuseError::ZeroAmount);
        }

        for (token_id, amount) in transfer
            .token_ids
            .iter()
            .zip(transfer.amounts.iter().map(|a| a.0))
        {
            if amount == 0 {
                return Err(DefuseError::ZeroAmount);
            }
            let token_id: TokenId = token_id.parse()?;

            self.internal_withdraw(sender_id.to_owned(), token_id.clone(), amount)
                .ok_or(DefuseError::BalanceOverflow)?;
            self.internal_deposit(transfer.receiver_id.clone(), token_id, amount)
                .ok_or(DefuseError::BalanceOverflow)?;
        }

        Ok(())
    }

    fn ft_withdraw(&mut self, owner_id: AccountId, withdraw: FtWithdraw) -> Result<()> {
        if withdraw.amount.0 == 0 {
            return Err(DefuseError::ZeroAmount);
        }

        if let Some(storage_deposit) = withdraw.storage_deposit {
            self.internal_withdraw(
                owner_id.clone(),
                TokenId::Nep141(self.wnear_id().into_owned()),
                storage_deposit.as_yoctonear(),
            )
            .ok_or(DefuseError::BalanceOverflow)?;
        }
        self.internal_withdraw(owner_id, TokenId::Nep141(withdraw.token), withdraw.amount.0)
            .ok_or(DefuseError::BalanceOverflow)?;

        Ok(())
    }

    fn nft_withdraw(&mut self, owner_id: AccountId, withdraw: NftWithdraw) -> Result<()> {
        if let Some(storage_deposit) = withdraw.storage_deposit {
            self.internal_withdraw(
                owner_id.clone(),
                TokenId::Nep141(self.wnear_id().into_owned()),
                storage_deposit.as_yoctonear(),
            )
            .ok_or(DefuseError::BalanceOverflow)?;
        }
        self.internal_withdraw(
            owner_id,
            TokenId::Nep171(withdraw.token, withdraw.token_id),
            1,
        )
        .ok_or(DefuseError::BalanceOverflow)?;

        Ok(())
    }

    fn mt_withdraw(&mut self, owner_id: AccountId, withdraw: MtWithdraw) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Default, Clone)]
pub struct CachedAccounts(HashMap<AccountId, CachedAccount>);

impl CachedAccounts {
    #[inline]
    pub fn get(&self, account_id: &AccountIdRef) -> Option<&CachedAccount> {
        self.0.get(account_id)
    }

    #[inline]
    pub fn get_mut(&mut self, account_id: &AccountIdRef) -> Option<&mut CachedAccount> {
        self.0.get_mut(account_id)
    }

    #[inline]
    pub fn get_or_create(&mut self, account_id: AccountId) -> &mut CachedAccount {
        self.0.entry(account_id).or_default()
    }
}

#[derive(Debug, Clone, Default)]
pub struct CachedAccount {
    nonces: Nonces<HashMap<U248, U256>>,

    public_keys_added: HashSet<PublicKey>,
    public_keys_removed: HashSet<PublicKey>,

    token_amounts: TokenAmounts<HashMap<TokenId, u128>>,
}

impl CachedAccount {
    #[inline]
    pub fn is_nonce_used(&self, nonce: U256) -> bool {
        self.nonces.is_used(nonce)
    }

    #[inline]
    pub fn commit_nonce(&mut self, n: U256) -> bool {
        self.nonces.commit(n)
    }
}