use core::{
    fmt::{self, Debug, Display},
    str::FromStr,
};

use near_account_id::AccountType;
use near_sdk::{env, near, AccountId, AccountIdRef, CurveType};
use serde_with::serde_as;

use super::{AsCurve, Curve, Ed25519, ParseCurveError, Secp256k1};

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
pub enum PublicKey {
    Ed25519(#[serde_as(as = "AsCurve<Ed25519>")] <Ed25519 as Curve>::PublicKey),
    Secp256k1(#[serde_as(as = "AsCurve<Secp256k1>")] <Secp256k1 as Curve>::PublicKey),
}

impl PublicKey {
    #[inline]
    pub const fn curve_type(&self) -> CurveType {
        match self {
            Self::Ed25519(_) => CurveType::ED25519,
            Self::Secp256k1(_) => CurveType::SECP256K1,
        }
    }

    #[inline]
    pub fn to_implicit_account_id(&self) -> AccountId {
        match self {
            Self::Ed25519(pk) => hex::encode(pk),
            Self::Secp256k1(pk) => {
                // https://ethereum.org/en/developers/docs/accounts/#account-creation
                format!("0x{}", hex::encode(&env::keccak256_array(pk)[12..]))
            }
        }
        .try_into()
        .unwrap_or_else(|_| unreachable!())
    }

    #[inline]
    pub fn from_implicit_account_id(account_id: impl AsRef<AccountIdRef>) -> Option<Self> {
        Some(account_id.as_ref())
            .filter(|account_id| {
                matches!(
                    account_id.get_account_type(),
                    AccountType::NearImplicitAccount
                )
            })
            .and_then(|a| hex::decode(a.as_str()).ok())
            .and_then(|bytes| bytes.try_into().ok())
            .map(Self::Ed25519)
    }
}

impl Debug for PublicKey {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&match self {
            PublicKey::Ed25519(pk) => Ed25519::to_base58(pk),
            PublicKey::Secp256k1(pk) => Secp256k1::to_base58(pk),
        })
    }
}

impl Display for PublicKey {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl FromStr for PublicKey {
    type Err = ParseCurveError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Ed25519::parse_base58(s) {
            Ok(pk) => Ok(Self::Ed25519(pk)),
            Err(ParseCurveError::InvalidCurveType) => {
                Secp256k1::parse_base58(s).map(Self::Secp256k1)
            }
            Err(err) => Err(err),
        }
    }
}

#[cfg(all(feature = "abi", not(target_arch = "wasm32")))]
mod abi {
    use super::*;

    impl PublicKey {
        pub(super) fn example_ed25519() -> Self {
            "ed25519:5TagutioHgKLh7KZ1VEFBYfgRkPtqnKm9LoMnJMJugxm"
                .parse()
                .unwrap()
        }

        pub(super) fn example_secp256k1() -> Self {
            "secp256k1:5KN6ZfGZgH1puWwH1Nc1P8xyrFZSPHDw3WUP6iitsjCECJLrGBq"
                .parse()
                .unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secp256k1() {
        let pk: PublicKey = "secp256k1:62dVbWHiN4JCcWpetYs2zEqHYoDzhz5Eb1bEhqWxTzEdor2ndPM1WCrkJSr1911uANxZLezQwEaxMaywqMc6jPSM".parse().unwrap();
        assert_eq!(
            pk.to_implicit_account_id(),
            "0xd63b006b0cfd2fe3ab95db515cd59e519f92fe55"
                .parse::<AccountId>()
                .unwrap()
        );
    }
}
