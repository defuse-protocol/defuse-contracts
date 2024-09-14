mod differ;

use defuse_contracts::defuse::{verify::diff::SignedDiffs, DefuseError};
use differ::Differ;

use crate::DefuseImpl;

impl DefuseImpl {
    pub(crate) fn apply_signed_diffs(&mut self, diffs: SignedDiffs) -> Result<(), DefuseError> {
        let mut differ = Differ::default();

        for (account_id, signed_diffs) in diffs {
            let mut account = self.accounts.get_or_create_fresh(account_id.clone());

            for signed in signed_diffs {
                let diff = account.verify_signed_as_nep413(&account_id, signed)?;

                differ.commit_account_diff(&mut account.state, diff)?;
            }
        }

        differ.ensure_invariant()
    }
}
