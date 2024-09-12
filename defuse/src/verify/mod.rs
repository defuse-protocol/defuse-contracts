pub mod diff;

use std::collections::HashSet;

use defuse_contracts::{
    crypto::PublicKey,
    defuse::verify::{diff::SignedDiffs, Verifier},
};
use near_sdk::{env, near, AccountId};

use crate::{accounts::TypedAccount, DefuseImpl, DefuseImplExt};

#[near]
impl Verifier for DefuseImpl {
    fn public_keys_of(&self, account_id: Option<AccountId>) -> HashSet<PublicKey> {
        self.accounts
            .get(&account_id.unwrap_or_else(env::predecessor_account_id))
            .and_then(TypedAccount::as_named)
            .expect("only named accounts can have multiple public keys")
            .iter_public_keys()
            .copied()
            .collect()
    }

    fn add_public_key(&mut self, public_key: PublicKey) -> bool {
        self.accounts
            .get_or_insert(env::predecessor_account_id())
            .as_named_mut()
            .expect("only named accounts can add multiplt public keys")
            .add_public_key(public_key)
    }

    fn remove_public_key(&mut self, public_key: &PublicKey) -> bool {
        self.accounts
            .get_mut(&env::predecessor_account_id())
            .and_then(TypedAccount::as_named_mut)
            .expect("only named accounts can have multiple public keys")
            .remove_public_key(public_key)
    }

    fn apply_signed_diffs(&mut self, diffs: SignedDiffs) {
        self.apply_signed_diffs(diffs).unwrap()
    }
}
