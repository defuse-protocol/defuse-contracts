use std::{borrow::Cow, collections::HashSet};

use near_sdk::{ext_contract, near, AccountId};

use crate::{crypto::PublicKey, nep413::U256, utils::serde::wrappers::Base64};

#[ext_contract(ext_public_key_manager)]
pub trait AccountManager {
    /// Check if account has given public key
    fn has_public_key(&self, account_id: &AccountId, public_key: &PublicKey) -> bool;

    /// Returns set of public keys registered for given account
    fn public_keys_of(&self, account_id: &AccountId) -> HashSet<PublicKey>;

    /// Registers or re-activates `public_key` under the caller account_id.
    ///
    /// NOTE: MUST attach 1 yⓃ for security purposes.
    fn add_public_key(&mut self, public_key: PublicKey);

    /// Deactivate `public_key` from the caller account_id,
    /// i.e. this key can't be used to make any actions unless it's re-created.
    ///
    /// NOTE: MUST attach 1 yⓃ for security purposes.
    fn remove_public_key(&mut self, public_key: &PublicKey);

    /// Returns whether given nonce was already used by the account
    /// NOTE: nonces are non-sequential and follow
    /// [permit2 nonce schema](https://docs.uniswap.org/contracts/permit2/reference/signature-transfer#nonce-schema).
    fn is_nonce_used(&self, account_id: &AccountId, nonce: Base64<U256>) -> bool;

    /// NOTE: MUST attach 1 yⓃ for security purposes.
    fn invalidate_nonces(&mut self, nonces: Vec<Base64<U256>>);
}

#[must_use = "make sure to `.emit()` this event"]
#[near(serializers = [json])]
#[derive(Debug)]
pub struct PublicKeyEvent<'a> {
    pub public_key: Cow<'a, PublicKey>,
}
