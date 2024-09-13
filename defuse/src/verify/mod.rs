pub mod diff;

use std::collections::HashSet;

use defuse_contracts::{
    crypto::PublicKey,
    defuse::verify::{diff::SignedDiffs, Verifier},
};
use near_sdk::{env, near, AccountId};

use crate::{DefuseImpl, DefuseImplExt};

#[near]
impl Verifier for DefuseImpl {
    fn public_keys_of(&self, account_id: Option<AccountId>) -> HashSet<PublicKey> {
        self.accounts
            .get(&account_id.unwrap_or_else(env::predecessor_account_id))
            .map(|account| account.iter_public_keys().copied().collect())
            .unwrap_or_default()
    }

    fn add_public_key(&mut self, public_key: PublicKey) {
        self.accounts
            .get_or_create(env::predecessor_account_id())
            .add_public_key(public_key);
    }

    fn remove_public_key(&mut self, public_key: &PublicKey) -> bool {
        let Some(account) = self.accounts.get_mut(&env::predecessor_account_id()) else {
            return false;
        };
        account.remove_public_key(public_key)
    }

    fn apply_signed_diffs(&mut self, diffs: SignedDiffs) {
        self.apply_signed_diffs(diffs).unwrap()
    }
}
