use core::marker::PhantomData;

use near_sdk::{
    bs58,
    serde::{de, Deserialize, Deserializer, Serialize, Serializer},
};
use serde_with::{DeserializeAs, SerializeAs};
use thiserror::Error as ThisError;

pub trait Curve {
    const PREFIX: &'static str;

    type PublicKey;
    type Signature;

    #[inline]
    fn to_base58(bytes: impl AsRef<[u8]>) -> String {
        format!(
            "{}:{}",
            Self::PREFIX,
            bs58::encode(bytes.as_ref()).into_string()
        )
    }

    fn parse_base58<const N: usize>(s: impl AsRef<str>) -> Result<[u8; N], ParseCurveError> {
        let s = s.as_ref();
        let data = if let Some((curve, data)) = s.split_once(':') {
            if !curve.eq_ignore_ascii_case(Self::PREFIX) {
                return Err(ParseCurveError::InvalidCurveType);
            }
            data
        } else {
            s
        };
        bs58::decode(data.as_bytes())
            .into_array_const()
            .map_err(Into::into)
    }
}

pub struct Ed25519;

impl Curve for Ed25519 {
    const PREFIX: &'static str = "ed25519";

    type PublicKey = [u8; 32];
    type Signature = [u8; 64];
}

pub struct Secp256k1;

impl Curve for Secp256k1 {
    const PREFIX: &'static str = "secp256k1";

    type PublicKey = [u8; 64];
    /// Concatenated `r`, `s` and `v`
    type Signature = [u8; 65];
}

#[derive(Debug, ThisError)]
pub enum ParseCurveError {
    #[error("invalid curve type")]
    InvalidCurveType,
    #[error("base58: {0}")]
    Base58(#[from] bs58::decode::Error),
}

pub struct AsCurve<C: Curve>(PhantomData<C>);

impl<C: Curve, const N: usize> SerializeAs<[u8; N]> for AsCurve<C> {
    fn serialize_as<S>(source: &[u8; N], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        C::to_base58(source).serialize(serializer)
    }
}

impl<'de, C: Curve, const N: usize> DeserializeAs<'de, [u8; N]> for AsCurve<C> {
    fn deserialize_as<D>(deserializer: D) -> Result<[u8; N], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <&str as Deserialize>::deserialize(deserializer)?;
        C::parse_base58(s).map_err(de::Error::custom)
    }
}

#[cfg(all(feature = "abi", not(target_arch = "wasm32")))]
mod abi {
    use super::*;

    use near_sdk::schemars::{
        gen::SchemaGenerator,
        schema::{InstanceType, Schema, SchemaObject, StringValidation},
        JsonSchema,
    };
    use serde_with::schemars_0_8::JsonSchemaAs;

    impl<C: Curve, const N: usize> JsonSchemaAs<[u8; N]> for AsCurve<C> {
        fn schema_name() -> String {
            String::schema_name()
        }

        fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
            SchemaObject {
                instance_type: Some(InstanceType::String.into()),
                string: Some(
                    StringValidation {
                        pattern: Some(format!("^{}:", C::PREFIX)),
                        ..Default::default()
                    }
                    .into(),
                ),
                extensions: [("contentEncoding", "base58".into())]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect(),
                ..Default::default()
            }
            .into()
        }

        fn is_referenceable() -> bool {
            false
        }
    }
}
