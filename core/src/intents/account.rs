use defuse_crypto::PublicKey;
use defuse_serde_utils::base64::Base64;
use near_sdk::{near, AccountIdRef};
use serde_with::serde_as;

use crate::{engine::State, DefuseError, Nonce, Result};

use super::ExecutableIntent;

/// Add public key to the signer account
#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct AddPublicKey {
    pub public_key: PublicKey,
}

impl ExecutableIntent for AddPublicKey {
    #[inline]
    fn execute_intent<S>(self, signer_id: &AccountIdRef, state: &mut S) -> Result<()>
    where
        S: State,
    {
        if !state.add_public_key(signer_id.to_owned(), self.public_key) {
            return Err(DefuseError::PublicKeyExists);
        }
        Ok(())
    }
}

/// Remove public key to the signer account
#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct RemovePublicKey {
    pub public_key: PublicKey,
}

impl ExecutableIntent for RemovePublicKey {
    #[inline]
    fn execute_intent<S>(self, signer_id: &AccountIdRef, state: &mut S) -> crate::Result<()>
    where
        S: State,
    {
        if !state.remove_public_key(signer_id.to_owned(), self.public_key) {
            return Err(DefuseError::PublicKeyNotExist);
        }
        Ok(())
    }
}

/// Invalidate given nonces TODO: error?
#[cfg_attr(
    all(feature = "abi", not(target_arch = "wasm32")),
    serde_as(schemars = true)
)]
#[cfg_attr(
    not(all(feature = "abi", not(target_arch = "wasm32"))),
    serde_as(schemars = false)
)]
#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct InvalidateNonces {
    #[serde_as(as = "Vec<Base64>")]
    pub nonces: Vec<Nonce>,
}

impl ExecutableIntent for InvalidateNonces {
    #[inline]
    fn execute_intent<S>(self, signer_id: &AccountIdRef, state: &mut S) -> crate::Result<()>
    where
        S: State,
    {
        for nonce in self.nonces {
            if !state.commit_nonce(signer_id.to_owned(), nonce) {
                return Err(DefuseError::NonceUsed);
            }
        }
        Ok(())
    }
}
