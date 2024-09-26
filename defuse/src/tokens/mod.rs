mod nep141;
mod nep171;
mod nep245;

use defuse_contracts::defuse::{tokens::TokenId, DefuseError, Result};
use near_sdk::{
    near,
    store::{iterable_map::Entry, IterableMap},
    AccountId, IntoStorageKey,
};

use crate::DefuseImpl;

impl DefuseImpl {
    pub(crate) fn internal_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: AccountId,
        token_amounts: Vec<(TokenId, u128)>,
        #[allow(unused_variables)] memo: Option<String>,
    ) -> Result<()> {
        // TODO: check sender != receiver
        // withdraw
        let sender = self
            .accounts
            .get_mut(sender_id)
            .ok_or(DefuseError::AccountNotFound)?;
        for (token_id, amount) in &token_amounts {
            sender.token_balances.withdraw(token_id.clone(), *amount)?;
        }

        // deposit
        let receiver = self.accounts.get_or_create(receiver_id);
        for (token_id, amount) in token_amounts {
            receiver.token_balances.deposit(token_id, amount)?;
        }

        // TODO: log transfer event with memo

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
    pub fn deposit(&mut self, token_id: TokenId, amount: u128) -> Result<u128> {
        Ok(match self.0.entry(token_id) {
            Entry::Vacant(_) if amount == 0 => 0,
            Entry::Vacant(entry) => *entry.insert(amount),
            Entry::Occupied(mut entry) => {
                let b = entry.get_mut();
                *b = b.checked_add(amount).ok_or(DefuseError::BalanceOverflow)?;
                if *b == 0 {
                    entry.remove()
                } else {
                    *b
                }
            }
        })
    }

    #[inline]
    pub fn withdraw(&mut self, token_id: TokenId, amount: u128) -> Result<u128>
where {
        Ok(match self.0.entry(token_id) {
            Entry::Vacant(_) if amount == 0 => 0,
            Entry::Vacant(entry) => *entry.insert(amount),
            Entry::Occupied(mut entry) => {
                let b = entry.get_mut();
                *b = b.checked_sub(amount).ok_or(DefuseError::BalanceOverflow)?;
                if *b == 0 {
                    entry.remove()
                } else {
                    *b
                }
            }
        })
    }

    #[inline]
    pub fn add_delta(&mut self, token_id: TokenId, delta: i128) -> Result<u128> {
        match self.0.entry(token_id) {
            Entry::Vacant(_) if delta < 0 => Err(DefuseError::BalanceOverflow),
            Entry::Vacant(_) if delta == 0 => Ok(0),
            Entry::Vacant(entry) => Ok(*entry.insert(delta.unsigned_abs())),
            Entry::Occupied(mut entry) => {
                let b = entry.get_mut();
                *b = b
                    .checked_add_signed(delta)
                    .ok_or(DefuseError::BalanceOverflow)?;
                Ok(if *b == 0 { entry.remove() } else { *b })
            }
        }
    }
}
