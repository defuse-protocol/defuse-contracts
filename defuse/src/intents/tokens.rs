use std::collections::HashMap;

use defuse_contracts::defuse::{
    tokens::{TokenAmounts, TokenId},
    DefuseError, Result,
};
use near_sdk::AccountId;

use crate::{accounts::Accounts, tokens::TokensBalances};

#[derive(Debug, Default)]
pub struct TokenTracker {
    deltas: TokenAmounts<i128>,
}

impl TokenTracker {
    #[inline]
    pub fn deposit(
        &mut self,
        balances: &mut TokensBalances,
        token_id: TokenId,
        amount: u128,
    ) -> Result<u128> {
        self.deltas.add(token_id.clone(), amount)?;
        balances.deposit(token_id.clone(), amount)
    }

    #[inline]
    pub fn withdraw(
        &mut self,
        balances: &mut TokensBalances,
        token_id: TokenId,
        amount: u128,
    ) -> Result<u128> {
        self.deltas.sub(token_id.clone(), amount)?;
        balances.withdraw(token_id.clone(), amount)
    }

    #[inline]
    pub fn add_delta(
        &mut self,
        balances: &mut TokensBalances,
        token_id: TokenId,
        delta: i128,
    ) -> Result<u128> {
        self.deltas.add(token_id.clone(), delta)?;
        balances.add_delta(token_id.clone(), delta)
    }

    #[inline]
    pub fn finalize(self) -> Result<()> {
        if !self.deltas.is_empty() {
            return Err(DefuseError::InvariantViolated);
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct TransferTracker {
    postponed_deposits: HashMap<AccountId, TokenAmounts<u128>>,
}

impl TransferTracker {
    pub fn transfer(
        &mut self,
        sender_balances: &mut TokensBalances,
        receiver_id: AccountId,
        tokens: impl IntoIterator<Item = (TokenId, u128)>,
    ) -> Result<()> {
        let receiver = self.postponed_deposits.entry(receiver_id).or_default();
        for (token_id, amount) in tokens {
            sender_balances.withdraw(token_id.clone(), amount)?;
            receiver.add(token_id, amount)?;
        }
        Ok(())
    }

    pub fn finalize(self, accounts: &mut Accounts) -> Result<()> {
        for (receiver_id, tokens) in self.postponed_deposits {
            let receiver = accounts.get_or_create(receiver_id);
            for (token_id, amount) in tokens {
                receiver.token_balances.deposit(token_id, amount)?;
            }
        }
        Ok(())
    }
}
