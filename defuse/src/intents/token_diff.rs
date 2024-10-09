use defuse_contracts::defuse::{intents::token_diff::TokenDiff, DefuseError, Result};
use near_sdk::AccountId;

use crate::accounts::Account;

use super::{IntentExecutor, State};

impl IntentExecutor<TokenDiff> for State {
    fn execute_intent(
        &mut self,
        _account_id: &AccountId,
        account: &mut Account,
        intent: TokenDiff,
    ) -> Result<()> {
        let fees_collected = self
            .runtime
            .postponed_deposits
            .entry(self.fee_collector.clone())
            .or_default();

        for (token_id, delta) in intent.diff {
            account.token_balances.add_delta(token_id.clone(), delta)?;

            let fee = self.fee.fee(delta.unsigned_abs());
            fees_collected.add(token_id.clone(), fee)?;

            self.runtime.total_supply_deltas.add(
                token_id.clone(),
                delta
                    .checked_add_unsigned(fee)
                    .ok_or(DefuseError::IntegerOverflow)?,
            )?;
        }

        Ok(())
    }
}
