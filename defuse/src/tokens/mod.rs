mod nep141;
mod nep171;
mod nep245;

use defuse_contracts::{
    defuse::{tokens::TokenId, DefuseError, Result},
    utils::cleanup::DefaultMap,
};
use near_sdk::{near, store::IterableMap, AccountId, IntoStorageKey};

use crate::DefuseImpl;

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

    pub(crate) fn internal_withdraw(
        &mut self,
        account_id: &AccountId,
        token_amounts: impl IntoIterator<Item = (TokenId, u128)>,
    ) -> Result<()> {
        let account = self
            .accounts
            .get_mut(account_id)
            .ok_or(DefuseError::AccountNotFound)?;
        for (token_id, amount) in token_amounts {
            account.token_balances.withdraw(token_id.clone(), amount)?;
            self.total_supplies.withdraw(token_id, amount)?;
        }
        Ok(())
    }

    pub(crate) fn internal_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: AccountId,
        token_amounts: Vec<(TokenId, u128)>,
        #[allow(unused_variables)] memo: Option<String>,
    ) -> Result<()> {
        if sender_id == &receiver_id {
            return Err(DefuseError::InvalidSenderReceiver);
        }
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
        let mut balance = self.0.entry_or_default(token_id);
        *balance = balance
            .checked_add(amount)
            .ok_or(DefuseError::BalanceOverflow)?;
        Ok(*balance)
    }

    #[inline]
    pub fn withdraw(&mut self, token_id: TokenId, amount: u128) -> Result<u128>
where {
        let mut balance = self.0.entry_or_default(token_id);
        *balance = balance
            .checked_sub(amount)
            .ok_or(DefuseError::BalanceOverflow)?;
        Ok(*balance)
    }

    #[inline]
    pub fn add_delta(&mut self, token_id: TokenId, delta: i128) -> Result<u128> {
        let mut balance = self.0.entry_or_default(token_id);
        *balance = balance
            .checked_add_signed(delta)
            .ok_or(DefuseError::BalanceOverflow)?;
        Ok(*balance)
    }
}
