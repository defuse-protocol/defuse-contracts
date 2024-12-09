use core::{
    fmt::{self, Debug, Display},
    str::FromStr,
};

use near_account_id::{AccountId, AccountIdRef, AccountType};
use near_sdk::{bs58, env, near};

use crate::{Curve, CurveType, Ed25519, ParseCurveError, Secp256k1};

#[near(serializers = [borsh])]
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(
    feature = "serde",
    derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)
)]
pub enum PublicKey {
    Ed25519(<Ed25519 as Curve>::PublicKey),
    Secp256k1(<Secp256k1 as Curve>::PublicKey),
}

impl PublicKey {
    #[inline]
    pub const fn curve_type(&self) -> CurveType {
        match self {
            Self::Ed25519(_) => CurveType::Ed25519,
            Self::Secp256k1(_) => CurveType::Secp256k1,
        }
    }

    #[inline]
    const fn data(&self) -> &[u8] {
        match self {
            Self::Ed25519(data) => data,
            Self::Secp256k1(data) => data,
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
    pub fn from_implicit_account_id(account_id: &AccountIdRef) -> Option<Self> {
        Some(account_id)
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
        write!(
            f,
            "{}:{}",
            self.curve_type(),
            bs58::encode(self.data()).into_string()
        )
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
        let (curve, data) = if let Some((curve, data)) = s.split_once(':') {
            (
                curve.parse().map_err(|_| ParseCurveError::WrongCurveType)?,
                data,
            )
        } else {
            (CurveType::Ed25519, s)
        };
        let decoder = bs58::decode(data.as_bytes());
        match curve {
            CurveType::Ed25519 => decoder.into_array_const().map(Self::Ed25519),
            CurveType::Secp256k1 => decoder.into_array_const().map(Self::Secp256k1),
        }
        .map_err(Into::into)
    }
}

#[cfg(all(feature = "abi", not(target_arch = "wasm32")))]
mod abi {
    use super::*;

    use near_sdk::{
        schemars::{
            gen::SchemaGenerator,
            schema::{InstanceType, Metadata, Schema, SchemaObject},
            JsonSchema,
        },
        serde_json,
    };

    impl JsonSchema for PublicKey {
        fn schema_name() -> String {
            String::schema_name()
        }

        fn is_referenceable() -> bool {
            false
        }

        fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
            SchemaObject {
                instance_type: Some(InstanceType::String.into()),
                extensions: [("contentEncoding", "base58".into())]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect(),
                metadata: Some(
                    Metadata {
                        examples: [Self::example_ed25519(), Self::example_secp256k1()]
                            .map(serde_json::to_value)
                            .map(Result::unwrap)
                            .into(),
                        ..Default::default()
                    }
                    .into(),
                ),
                ..Default::default()
            }
            .into()
        }
    }

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
