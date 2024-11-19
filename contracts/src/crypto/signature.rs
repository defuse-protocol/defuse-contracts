use core::{
    fmt::{self, Debug, Display},
    str::FromStr,
};

use near_sdk::near;
use serde_with::serde_as;

use super::{AsCurve, Curve, CurvePrefix, Ed25519, ParseCurveError, Secp256k1};

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
    schemars(example = "Self::example_ed25519", example = "Self::example_secp256k1")
)]
#[serde(untagged)]
pub enum Signature {
    Ed25519(#[serde_as(as = "AsCurve<Ed25519>")] <Ed25519 as Curve>::Signature),
    Secp256k1(#[serde_as(as = "AsCurve<Secp256k1>")] <Secp256k1 as Curve>::Signature),
}

impl Debug for Signature {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&match self {
            Self::Ed25519(pk) => Ed25519::to_base58(pk),
            Self::Secp256k1(pk) => Secp256k1::to_base58(pk),
        })
    }
}

impl Display for Signature {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl FromStr for Signature {
    type Err = ParseCurveError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Ed25519::parse_base58(s).map(Self::Ed25519) {
            Err(ParseCurveError::InvalidCurveType) => {}
            r => return r,
        }
        match Secp256k1::parse_base58(s).map(Self::Secp256k1) {
            Err(ParseCurveError::InvalidCurveType) => {}
            r => return r,
        }
        return Err(ParseCurveError::InvalidCurveType);
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

        pub(super) fn example_secp256k1() -> Self {
            Self::Secp256k1 {
                signature: Secp256k1::parse_base58(
                    "secp256k1:7huDZxNnibusy6wFkbUBQ9Rqq2VmCKgTWYdJwcPj8VnciHjZKPa41rn5n6WZnMqSUCGRHWMAsMjKGtMVVmpETCeCs")
                    .unwrap(),
            }
        }
    }
}
