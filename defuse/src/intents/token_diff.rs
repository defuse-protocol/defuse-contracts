use defuse_contracts::defuse::{intents::token_diff::TokenDiff, DefuseError, Result};
use near_sdk::AccountId;

use crate::accounts::Account;

use super::{runtime::IntentExecutor, Runtime};

impl<'a> IntentExecutor<TokenDiff> for Runtime<'a> {
    fn execute_intent(
        &mut self,
        _account_id: &AccountId,
        account: &mut Account,
        intent: TokenDiff,
    ) -> Result<()> {
        for (token_id, delta) in intent.diff {
            account.token_balances.add_delta(token_id.clone(), delta)?;

            let fee = self.fees.apply(delta.unsigned_abs());
            self.postponed_deposits
                .entry(self.fees.collector.clone())
                .or_default()
                .add(token_id.clone(), fee)?;

            self.total_supply_deltas.add(
                token_id.clone(),
                delta
                    .checked_add_unsigned(fee)
                    .ok_or(DefuseError::IntegerOverflow)?,
            )?;
        }

        Ok(())
    }
}
