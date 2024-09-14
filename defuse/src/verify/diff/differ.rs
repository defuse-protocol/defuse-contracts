use defuse_contracts::defuse::{
    tokens::TokenId,
    verify::diff::{tokens::TokenDeltas, AccountDiff},
    DefuseError,
};

use crate::{accounts::AccountState, tokens::TokensBalances};

#[derive(Debug, Default)]
pub struct Differ {
    token_deltas: TokenDeltas,
}

impl Differ {
    pub fn commit_account_diff(
        &mut self,
        state: &mut AccountState,
        diff: AccountDiff,
    ) -> Result<(), DefuseError> {
        if diff.deadline.has_expired() {
            return Err(DefuseError::DeadlineExpired);
        }

        self.commit_token_diff(&mut state.token_balances, diff.tokens)
        // TODO: log diff.query_id
    }

    fn commit_token_diff(
        &mut self,
        balances: &mut TokensBalances,
        deltas: TokenDeltas,
    ) -> Result<(), DefuseError> {
        for (token_id, delta) in deltas {
            self.on_token_delta(token_id.clone(), delta)?;

            if delta.is_negative() {
                balances.withdraw(&token_id, -delta as u128)?;
            } else {
                balances.deposit(token_id, delta as u128)?;
            }
        }
        Ok(())
    }

    #[inline]
    fn on_token_delta(&mut self, token_id: TokenId, delta: i128) -> Result<(), DefuseError> {
        self.token_deltas.add_delta(token_id, delta)?;
        Ok(())
    }

    #[inline]
    pub fn ensure_invariant(self) -> Result<(), DefuseError> {
        if !self.token_deltas.is_empty() {
            return Err(DefuseError::InvariantViolated);
        }
        Ok(())
    }
}
