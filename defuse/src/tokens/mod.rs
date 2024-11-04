mod nep141;
mod nep171;
mod nep245;

use std::borrow::Cow;

use defuse_contracts::{
    defuse::{tokens::TokenId, DefuseError, Result},
    nep245::{MtBurnEvent, MtEventEmit, MtMintEvent},
    utils::cleanup::DefaultMap,
};
use near_sdk::{
    json_types::U128, near, require, store::IterableMap, AccountId, Gas, IntoStorageKey,
};

use crate::{accounts::Account, state::State, DefuseImpl};

pub const STORAGE_DEPOSIT_GAS: Gas = Gas::from_tgas(10);

impl DefuseImpl {
    #[inline]
    pub(crate) fn internal_balance_of(&self, account_id: &AccountId, token_id: &TokenId) -> u128 {
        self.accounts
            .get(account_id)
            .map(|account| account.token_balances.balance_of(token_id))
            .unwrap_or_default()
    }

    #[inline]
    pub(crate) fn internal_deposit(
        &mut self,
        account_id: AccountId,
        token_amounts: impl IntoIterator<Item = (TokenId, u128)>,
        memo: Option<&str>,
    ) -> Result<()> {
        let account = self.accounts.get_or_create(account_id.clone());
        self.state
            .internal_deposit(&account_id, account, token_amounts, memo)
    }
}

impl State {
    pub fn internal_deposit(
        &mut self,
        owner_id: &AccountId,
        owner: &mut Account,
        token_amounts: impl IntoIterator<Item = (TokenId, u128)>,
        memo: Option<&str>,
    ) -> Result<()> {
        let mut event = MtMintEvent {
            owner_id: Cow::Borrowed(owner_id.as_ref()),
            token_ids: Default::default(),
            amounts: Default::default(),
            memo: memo.map(Into::into),
        };

        for (token_id, amount) in token_amounts {
            require!(amount > 0, "zero amount");

            event.token_ids.to_mut().push(token_id.to_string());
            event.amounts.to_mut().push(U128(amount));

            self.total_supplies.deposit(token_id.clone(), amount)?;
            owner.token_balances.deposit(token_id, amount)?;
        }

        [event].emit();

        Ok(())
    }

    pub fn internal_withdraw(
        &mut self,
        owner_id: &AccountId,
        owner: &mut Account,
        token_amounts: impl IntoIterator<Item = (TokenId, u128)>,
        memo: Option<&str>,
    ) -> Result<()> {
        let mut event = MtBurnEvent {
            owner_id: Cow::Borrowed(owner_id.as_ref()),
            authorized_id: None,
            token_ids: Default::default(),
            amounts: Default::default(),
            memo: memo.map(Into::into),
        };

        for (token_id, amount) in token_amounts {
            require!(amount > 0, "zero amount");

            event.token_ids.to_mut().push(token_id.to_string());
            event.amounts.to_mut().push(U128(amount));

            owner.token_balances.withdraw(token_id.clone(), amount)?;
            self.total_supplies.withdraw(token_id, amount)?;
        }

        [event].emit();

        Ok(())
    }
}

#[derive(Debug)]
#[near(serializers = [borsh])]
pub struct TokensBalances(IterableMap<TokenId, u128>);

impl TokensBalances {
    #[must_use]
    #[inline]
    pub fn new<S>(prefix: S) -> Self
    where
        S: IntoStorageKey,
    {
        Self(IterableMap::new(prefix))
    }

    #[must_use]
    #[inline]
    pub fn contains(&self, token_id: &TokenId) -> bool {
        self.0.contains_key(token_id)
    }

    #[must_use]
    #[inline]
    pub fn balance_of(&self, token_id: &TokenId) -> u128 {
        self.0.get(token_id).copied().unwrap_or_default()
    }

    #[inline]
    fn try_apply<E>(
        &mut self,
        token_id: TokenId,
        f: impl FnOnce(u128) -> Result<u128, E>,
    ) -> Result<u128, E> {
        let mut d = self.0.entry_or_default(token_id);
        *d = f(*d)?;
        Ok(*d)
    }

    #[inline]
    pub fn deposit(&mut self, token_id: TokenId, amount: u128) -> Result<u128> {
        self.try_apply(token_id, |b| {
            b.checked_add(amount).ok_or(DefuseError::BalanceOverflow)
        })
    }

    #[inline]
    pub fn withdraw(&mut self, token_id: TokenId, amount: u128) -> Result<u128>
where {
        self.try_apply(token_id, |b| {
            b.checked_sub(amount).ok_or(DefuseError::BalanceOverflow)
        })
    }

    #[inline]
    pub fn add_delta(&mut self, token_id: TokenId, delta: i128) -> Result<u128> {
        self.try_apply(token_id, |b| {
            b.checked_add_signed(delta)
                .ok_or(DefuseError::BalanceOverflow)
        })
    }
}
