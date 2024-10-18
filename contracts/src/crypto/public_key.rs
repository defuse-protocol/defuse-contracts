use core::{
    fmt::{self, Debug, Display},
    str::FromStr,
};

use near_sdk::{bs58, env, near, AccountId, CurveType};
use serde_with::{DeserializeFromStr, SerializeDisplay};

use super::{Curve, Ed25519, ParseCurveError, Secp256k1};

#[derive(
    Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, SerializeDisplay, DeserializeFromStr,
)]
#[near(serializers = [borsh])]
pub enum PublicKey {
    Ed25519(<Ed25519 as Curve>::PublicKey),
    Secp256k1(<Secp256k1 as Curve>::PublicKey),
}

impl PublicKey {
    #[inline]
    pub const fn curve_type(&self) -> CurveType {
        match self {
            Self::Ed25519(_) => CurveType::ED25519,
            Self::Secp256k1(_) => CurveType::SECP256K1,
        }
    }

    const fn curve_prefix(&self) -> &'static str {
        match self {
            Self::Ed25519(_) => Ed25519::PREFIX,
            Self::Secp256k1(_) => Secp256k1::PREFIX,
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
}

impl Debug for PublicKey {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}",
            self.curve_prefix(),
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
                if curve.eq_ignore_ascii_case(Ed25519::PREFIX) {
                    CurveType::ED25519
                } else if curve.eq_ignore_ascii_case(Secp256k1::PREFIX) {
                    CurveType::SECP256K1
                } else {
                    return Err(ParseCurveError::InvalidCurveType);
                },
                data.as_bytes(),
            )
        } else {
            // defaults to Ed25519
            (CurveType::ED25519, s.as_bytes())
        };

        match curve {
            CurveType::ED25519 => bs58::decode(data).into_array_const().map(Self::Ed25519),
            CurveType::SECP256K1 => bs58::decode(data).into_array_const().map(Self::Secp256k1),
        }
        .map_err(Into::into)
    }
}

#[cfg(all(feature = "abi", not(target_arch = "wasm32")))]
mod abi {
    use super::*;

    use near_sdk::schemars::{
        gen::SchemaGenerator,
        schema::{InstanceType, Schema, SchemaObject},
        JsonSchema,
    };

    impl JsonSchema for PublicKey {
        fn schema_name() -> String {
            stringify!(PublicKey).to_string()
        }

        fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
            SchemaObject {
                instance_type: Some(InstanceType::String.into()),
                extensions: [(
                    "examples",
                    [
                        "ed25519:5TagutioHgKLh7KZ1VEFBYfgRkPtqnKm9LoMnJMJugxm",
                        "secp256k1:5KN6ZfGZgH1puWwH1Nc1P8xyrFZSPHDw3WUP6iitsjCECJLrGBq",
                    ]
                    .into_iter()
                    .inspect(|s| {
                        s.parse::<Self>().unwrap();
                    })
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .into(),
                )]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
                ..Default::default()
            }
            .into()
        }
    }
}
