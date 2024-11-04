use std::collections::HashMap;

use defuse_contracts::defuse::{tokens::TokenAmounts, DefuseError, Result};
use near_sdk::AccountId;

use crate::accounts::Accounts;

#[derive(Debug, Default)]
pub struct RuntimeState {
    /// Deposits postponed until [`.finalize()`](Self::finalize)
    pub postponed_deposits: HashMap<AccountId, TokenAmounts<u128>>,
    /// Total supply delta for each token
    pub total_supply_deltas: TokenAmounts<i128>,
}

impl RuntimeState {
    #[inline]
    pub fn finalize(&mut self, accounts: &mut Accounts) -> Result<()> {
        if !self.total_supply_deltas.is_empty() {
            return Err(DefuseError::InvariantViolated);
        }
        for (receiver_id, tokens) in self.postponed_deposits.drain() {
            let receiver = accounts.get_or_create(receiver_id);
            for (token_id, amount) in tokens {
                receiver.token_balances.deposit(token_id, amount)?;
            }
        }
        Ok(())
    }
}
