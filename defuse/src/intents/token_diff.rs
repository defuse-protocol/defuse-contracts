use defuse_contracts::defuse::{intents::token_diff::TokenDiff, DefuseError, Result};
use near_sdk::AccountId;

use crate::accounts::Account;

use super::{runtime::IntentExecutor, Runtime};

impl IntentExecutor<TokenDiff> for Runtime {
    fn execute_intent(
        &mut self,
        _account_id: &AccountId,
        account: &mut Account,
        intent: TokenDiff,
        referral: Option<&AccountId>,
    ) -> Result<()> {
        for (token_id, delta) in intent.diff {
            self.tokens
                .add_delta(&mut account.token_balances, token_id.clone(), delta)?;
            self.fees
                .on_token_amount(token_id, delta.unsigned_abs(), referral.cloned())?;
        }

        Ok(())
    }
}
