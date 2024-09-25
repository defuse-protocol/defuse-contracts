use defuse_contracts::defuse::{
    diff::{tokens::TokenDeltas, AccountDiff, SignedDiffer, SignedDiffs},
    tokens::TokenId,
    DefuseError,
};

use near_sdk::near;

use crate::{accounts::AccountState, tokens::TokensBalances, DefuseImpl, DefuseImplExt};

#[near]
impl SignedDiffer for DefuseImpl {
    fn apply_signed_diffs(&mut self, diffs: SignedDiffs) {
        self.internal_apply_signed_diffs(diffs).unwrap()
    }
}

impl DefuseImpl {
    fn internal_apply_signed_diffs(&mut self, diffs: SignedDiffs) -> Result<(), DefuseError> {
        let mut differ = Differ::default();

        for (account_id, signed_diffs) in diffs {
            let account = self.accounts.get_or_create(account_id.clone());

            for signed in signed_diffs {
                let diff = account.verify_signed_as_nep413(&account_id, signed)?;

                differ.commit_account_diff(&mut account.state, diff)?;
            }
        }

        differ.ensure_invariant()
    }
}

#[derive(Debug, Default)]
struct Differ {
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

            if delta.is_positive() {
                balances.deposit(token_id.clone(), delta as u128)?;
            } else {
                // TODO: overflows
                balances.withdraw(&token_id, -delta as u128)?;
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
