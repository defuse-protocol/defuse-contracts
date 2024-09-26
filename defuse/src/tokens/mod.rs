mod nep141;
mod nep171;
mod nep245;

use defuse_contracts::defuse::{tokens::TokenId, DefuseError, Result};
use near_sdk::{near, store::IterableMap, AccountId, IntoStorageKey};

use crate::DefuseImpl;

impl DefuseImpl {
    pub(crate) fn internal_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        token_id: TokenId,
        amount: u128,
    ) -> Result<()> {
        // TODO: check sender != receiver
        self.accounts
            .get_mut(sender_id)
            .ok_or(DefuseError::AccountNotFound)?
            .token_balances
            .withdraw(&token_id, amount)?;

        // TODO: get_or_create(recipient), but then we should public_key?
        self.accounts
            .get_mut(receiver_id)
            .ok_or(DefuseError::AccountNotFound)?
            .token_balances
            .deposit(token_id, amount)?;

        // TODO: log transfer event

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
    pub fn balance_of(&self, token_id: &TokenId) -> u128 {
        self.0.get(token_id).copied().unwrap_or_default()
    }

    #[inline]
    pub fn deposit(&mut self, token_id: TokenId, amount: u128) -> Result<()> {
        if amount == 0 {
            return Ok(());
        }
        let balance = self.0.entry(token_id).or_default();
        *balance = balance
            .checked_add(amount)
            .ok_or(DefuseError::BalanceOverflow)?;
        // TODO: emit log
        Ok(())
    }

    #[inline]
    pub fn withdraw(&mut self, token_id: &TokenId, amount: u128) -> Result<()>
where {
        if amount == 0 {
            return Ok(());
        }
        let balance = self
            .0
            .get_mut(token_id)
            .ok_or(DefuseError::BalanceOverflow)?;
        *balance = balance
            .checked_sub(amount)
            .ok_or(DefuseError::BalanceOverflow)?;
        // TODO: emit log
        Ok(())
    }
}
