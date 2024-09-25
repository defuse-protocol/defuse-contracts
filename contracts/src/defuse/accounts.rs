use std::collections::HashSet;

use near_sdk::{ext_contract, AccountId};

use crate::{crypto::PublicKey, nep413::Nonce, utils::serde::wrappers::DisplayFromStr};

#[ext_contract(ext_public_key_manager)]
pub trait AccountManager {
    /// Returns set of public keys registered for given account
    fn public_keys_of(&self, account_id: &AccountId) -> HashSet<PublicKey>;

    /// Registers given `public_key` under the caller account_id.
    fn add_public_key(&mut self, public_key: PublicKey);

    /// Removes given `public_key` from the caller account_id.
    fn remove_public_key(&mut self, public_key: &PublicKey) -> bool;

    /// Returns the first nonce available for given `public_key` of given `account_id`
    /// starting from `start` or `0` otherwise.
    ///
    /// NOTE: nonces are non-sequential and follow
    /// [permit2 nonce schema](https://docs.uniswap.org/contracts/permit2/reference/signature-transfer#nonce-schema).
    /// But using them sequentially is more storage-efficient.
    fn next_nonce_available(
        &self,
        account_id: &AccountId,
        public_key: &PublicKey,
        start: Option<DisplayFromStr<Nonce>>,
    ) -> Option<DisplayFromStr<Nonce>>;
}
