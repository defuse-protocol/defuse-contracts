use near_sdk::{env, near};
use serde_with::serde_as;

use super::PublicKey;

use crate::utils::serde::base64::Base64;

#[derive(Debug, Clone)]
#[cfg_attr(
    all(feature = "abi", not(target_arch = "wasm32")),
    serde_as(schemars = true)
)]
#[cfg_attr(
    not(all(feature = "abi", not(target_arch = "wasm32"))),
    serde_as(schemars = false)
)]
#[near(serializers = [borsh, json])]
#[serde(untagged)]
pub enum Signature {
    Ed25519 {
        #[serde_as(as = "Base64")]
        signature: [u8; 64],
        #[serde_as(as = "Base64")]
        public_key: [u8; 32],
    },
    Secp256k1 {
        /// Concatenated `r`, `s` and `v`
        #[serde_as(as = "Base64")]
        signature: [u8; 65],
    },
    // TODO: Secp256k1Compressed
}

impl Signature {
    /// Veirify the signature and return the public counterpart of the key
    /// that was used to sign given hash or `None` if the signature is
    /// invalid
    #[must_use]
    #[inline]
    pub fn verify(&self, hash: &[u8; 32]) -> Option<PublicKey> {
        match self {
            Signature::Ed25519 {
                ref signature,
                ref public_key,
            } => env::ed25519_verify(signature, hash, public_key)
                .then_some(PublicKey::Ed25519(*public_key)),
            Signature::Secp256k1 {
                signature: [signature @ .., v],
            } => {
                // TODO: are we sure about malleability_flag?
                // https://github.com/near/nearcore/blob/541c84a56bf9a2920271881f1a7804e2dd2d4ee7/core/crypto/src/signature.rs#L448
                env::ecrecover(hash, signature, *v, true).map(PublicKey::Secp256k1)
            }
        }
    }
}
