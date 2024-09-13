use defuse_contracts::defuse::{tokens::TokenId, DefuseError};
use near_sdk::{near, store::IterableMap, IntoStorageKey};

mod nep141;
mod nep171;

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
    pub fn deposit(&mut self, token_id: TokenId, amount: u128) -> Result<(), DefuseError> {
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
    pub fn withdraw(&mut self, token_id: &TokenId, amount: u128) -> Result<(), DefuseError>
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
