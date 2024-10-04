use std::collections::HashSet;

use near_sdk::{ext_contract, AccountId};

use crate::{crypto::PublicKey, nep413::Nonce, utils::serde::wrappers::DisplayFromStr};

use super::Result;

#[ext_contract(ext_public_key_manager)]
pub trait AccountManager {
    /// Check if account has given public key
    fn has_public_key(&self, account_id: &AccountId, public_key: &PublicKey) -> bool;

    /// Returns set of public keys registered for given account
    fn public_keys_of(&self, account_id: &AccountId) -> HashSet<PublicKey>;

    /// Registers or re-activates `public_key` under the caller account_id.
    fn add_public_key(&mut self, public_key: PublicKey);

    /// Deactivate `public_key` from the caller account_id,
    /// i.e. this key can't be used to make any actions unless it's re-created.
    fn remove_public_key(&mut self, public_key: &PublicKey);

    /// Returns whether given nonce was already used by the account
    fn is_nonce_used(&self, account_id: &AccountId, nonce: DisplayFromStr<Nonce>) -> bool;

    /// Returns the first nonce available for given `public_key` of given `account_id`
    /// starting from `start` or `0` otherwise.
    ///
    /// NOTE: nonces are non-sequential and follow
    /// [permit2 nonce schema](https://docs.uniswap.org/contracts/permit2/reference/signature-transfer#nonce-schema).
    /// But using them sequentially is more storage-efficient.
    fn find_unused_nonce(
        &self,
        account_id: &AccountId,
        start: Option<DisplayFromStr<Nonce>>,
    ) -> Option<DisplayFromStr<Nonce>>;

    #[handle_result]
    fn invalidate_nonce(&mut self, nonce: DisplayFromStr<Nonce>) -> Result<()>;
}
