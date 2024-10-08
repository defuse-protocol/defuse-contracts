mod nep141;
mod nep171;
mod nep245;

use defuse_contracts::{
    defuse::{tokens::TokenId, DefuseError, Result},
    utils::cleanup::DefaultMap,
};
use near_sdk::{near, store::IterableMap, AccountId, IntoStorageKey};

use crate::{accounts::Account, intents::runtime::Runtime, DefuseImpl};

impl DefuseImpl {
    pub(crate) fn internal_balance_of(&self, account_id: &AccountId, token_id: &TokenId) -> u128 {
        self.accounts
            .get(account_id)
            .map(|account| account.token_balances.balance_of(token_id))
            .unwrap_or_default()
    }

    pub(crate) fn internal_deposit(
        &mut self,
        account_id: AccountId,
        token_amounts: impl IntoIterator<Item = (TokenId, u128)>,
    ) -> Result<()> {
        let account = self.accounts.get_or_create(account_id);
        for (token_id, amount) in token_amounts {
            self.total_supplies.deposit(token_id.clone(), amount)?;
            account.token_balances.deposit(token_id, amount)?;
        }
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
            b.checked_add(amount).ok_or(DefuseError::IntegerOverflow)
        })
    }

    #[inline]
    pub fn withdraw(&mut self, token_id: TokenId, amount: u128) -> Result<u128>
where {
        self.try_apply(token_id, |b| {
            b.checked_sub(amount).ok_or(DefuseError::IntegerOverflow)
        })
    }

    #[inline]
    pub fn add_delta(&mut self, token_id: TokenId, delta: i128) -> Result<u128> {
        self.try_apply(token_id, |b| {
            b.checked_add_signed(delta)
                .ok_or(DefuseError::IntegerOverflow)
        })
    }
}

impl<'a> Runtime<'a> {
    pub fn internal_withdraw(
        &mut self,
        account: &mut Account,
        token_amounts: impl IntoIterator<Item = (TokenId, u128)>,
    ) -> Result<()> {
        for (token_id, amount) in token_amounts {
            if amount == 0 {
                return Err(DefuseError::ZeroAmount);
            }
            account.token_balances.withdraw(token_id.clone(), amount)?;
            self.total_supplies.withdraw(token_id, amount)?;
        }
        Ok(())
    }
}
