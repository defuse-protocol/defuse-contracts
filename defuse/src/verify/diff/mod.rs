mod invariant;

use defuse_contracts::defuse::{
    verify::diff::{tokens::TokenDeltas, AccountDiff, SignedDiffs},
    DefuseError,
};
use invariant::Invariant;

use crate::{accounts::AccountState, tokens::TokensBalances, DefuseImpl};

impl DefuseImpl {
    pub(crate) fn apply_signed_diffs(&mut self, diffs: SignedDiffs) -> Result<(), DefuseError> {
        let mut invariant = Invariant::default();

        for (account_id, signed_diffs) in diffs {
            let account = self.accounts.get_or_insert(account_id.clone());

            for signed in signed_diffs {
                let diff = account.verify_nep413(&account_id, signed)?;

                Self::commit_account_diff(&mut account.state, diff, &mut invariant)?;
            }
        }

        invariant.ensure()
    }

    fn commit_account_diff(
        state: &mut AccountState,
        diff: AccountDiff,
        invariant: &mut Invariant,
    ) -> Result<(), DefuseError> {
        if diff.deadline.has_expired() {
            return Err(DefuseError::DeadlineExpired);
        }

        Self::commit_token_diff(&mut state.token_balances, diff.tokens, invariant)

        // TODO: log diff.query_id
    }

    fn commit_token_diff(
        balances: &mut TokensBalances,
        deltas: TokenDeltas,
        invariant: &mut Invariant,
    ) -> Result<(), DefuseError> {
        for (token_id, delta) in deltas {
            invariant.on_token_delta(token_id.clone(), delta)?;

            if delta.is_negative() {
                balances.withdraw(&token_id, -delta as u128)?;
            } else {
                balances.deposit(token_id, delta as u128)?;
            }
        }
        Ok(())
    }
}
