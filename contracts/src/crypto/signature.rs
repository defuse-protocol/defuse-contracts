use near_sdk::{env, near};
use serde_with::serde_as;

use super::{AsCurve, Curve, Ed25519, PublicKey, Secp256k1};

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
#[cfg_attr(
    all(feature = "abi", not(target_arch = "wasm32")),
    schemars(example = "Self::example_ed25519")
)]
#[serde(untagged)]
pub enum Signature {
    /// Ed25519
    Ed25519 {
        #[serde_as(as = "AsCurve<Ed25519>")]
        signature: <Ed25519 as Curve>::Signature,

        #[serde_as(as = "AsCurve<Ed25519>")]
        public_key: <Ed25519 as Curve>::PublicKey,
    },
    /// Secp256k1
    Secp256k1 {
        #[serde_as(as = "AsCurve<Secp256k1>")]
        signature: <Secp256k1 as Curve>::Signature,
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
                // Note: Ethereum clients shift the recovery byte and this
                // logic might depend on chain id, so clients must rollback
                // these changes to v ∈ {0, 1}.
                // References:
                // * https://github.com/ethereumjs/ethereumjs-monorepo/blob/dc7169c16df6d36adeb6e234fcc66eb6cfc5ea3f/packages/util/src/signature.ts#L31-L62
                // * https://github.com/ethereum/go-ethereum/issues/19751#issuecomment-504900739
                env::ecrecover(
                    hash, signature, *v,
                    // Do not accept malleabile signatures:
                    // https://github.com/near/nearcore/blob/d73041cc1d1a70af4456fceefaceb1bf7f684fde/core/crypto/src/signature.rs#L448-L455
                    true,
                )
                .map(PublicKey::Secp256k1)
            }
        }
    }
}

#[cfg(all(feature = "abi", not(target_arch = "wasm32")))]
mod abi {
    use super::*;

    impl Signature {
        pub(super) fn example_ed25519() -> Self {
            Self::Ed25519 {
                signature: Ed25519::parse_base58(
                    "ed25519:DNxoVu7L7sHr9pcHGWQoJtPsrwheB8akht1JxaGpc9hGrpehdycXBMLJg4ph1bQ9bXdfoxJCbbwxj3Bdrda52eF")
                    .unwrap(),
                public_key: Ed25519::parse_base58("ed25519:5TagutioHgKLh7KZ1VEFBYfgRkPtqnKm9LoMnJMJugxm")
                    .unwrap(),
            }
        }
    }
}
