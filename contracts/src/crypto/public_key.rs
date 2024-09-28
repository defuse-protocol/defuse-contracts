use core::{
    fmt::{self, Debug, Display},
    str::FromStr,
};

use near_sdk::{bs58, env, near, AccountId};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use strum::{EnumDiscriminants, EnumString};
use thiserror::Error as ThisError;

#[derive(
    Clone,
    Copy,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    EnumDiscriminants,
    SerializeDisplay,
    DeserializeFromStr,
)]
#[strum_discriminants(
    name(PublicKeyType),
    derive(strum::Display, EnumString),
    strum(serialize_all = "snake_case")
)]
#[near(serializers = [borsh])]
pub enum PublicKey {
    Ed25519([u8; 32]),
    Secp256k1([u8; 64]),
}

impl PublicKey {
    #[inline]
    pub fn typ(&self) -> PublicKeyType {
        self.into()
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
            self.typ(),
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
    type Err = ParseKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (typ, data) = if let Some((typ, data)) = s.split_once(':') {
            (typ.parse()?, data.as_bytes())
        } else {
            // defaults to Ed25519
            (PublicKeyType::Ed25519, s.as_bytes())
        };

        match typ {
            PublicKeyType::Ed25519 => bs58::decode(data).into_array_const().map(Self::Ed25519),
            PublicKeyType::Secp256k1 => bs58::decode(data).into_array_const().map(Self::Secp256k1),
        }
        .map_err(Into::into)
    }
}

#[cfg(all(feature = "abi", not(target_arch = "wasm32")))]
mod abi {
    use super::*;

    use near_sdk::schemars::{gen::SchemaGenerator, schema::Schema, JsonSchema};

    impl JsonSchema for PublicKey {
        fn schema_name() -> String {
            String::schema_name()
        }

        fn json_schema(gen: &mut SchemaGenerator) -> Schema {
            String::json_schema(gen)
        }

        fn is_referenceable() -> bool {
            false
        }
    }
}

#[derive(Debug, ThisError)]
pub enum ParseKeyError {
    #[error("key type: '{0}'")]
    KeyType(#[from] strum::ParseError),
    #[error("base58: {0}")]
    Base58(#[from] bs58::decode::Error),
}
