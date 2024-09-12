use std::collections::HashSet;

use diff::SignedDiffs;
use near_sdk::{ext_contract, AccountId};

use crate::crypto::PublicKey;

pub mod diff;
pub mod payload;

#[ext_contract(ext_verifier)]
pub trait Verifier {
    fn public_keys_of(&self, account_id: Option<AccountId>) -> HashSet<PublicKey>;
    fn add_public_key(&mut self, public_key: PublicKey) -> bool;
    fn remove_public_key(&mut self, public_key: &PublicKey) -> bool;

    fn apply_signed_diffs(&mut self, diffs: SignedDiffs);
}
