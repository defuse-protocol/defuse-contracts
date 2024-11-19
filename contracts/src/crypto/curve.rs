use core::marker::PhantomData;

use near_sdk::{
    bs58, env,
    serde::{de, Deserialize, Deserializer, Serialize, Serializer},
    CryptoHash,
};
use serde_with::{DeserializeAs, SerializeAs};
use thiserror::Error as ThisError;

use super::{PublicKey, Signature};

pub trait Curve {
    type PublicKey;
    type Signature;
}

pub trait CurvePrefix: Curve {
    const PREFIX: &str;

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
    type PublicKey = [u8; 32];
    type Signature = [u8; 64];
}

impl CurvePrefix for Ed25519 {
    const PREFIX: &str = "ed25519";
}

pub struct Secp256k1;

impl Curve for Secp256k1 {
    type PublicKey = [u8; 64];
    /// Concatenated `r`, `s` and `v`
    type Signature = [u8; 65];
}

impl CurvePrefix for Secp256k1 {
    const PREFIX: &str = "secp256k1";
}

/// Note: Ethereum clients shift the recovery byte and this
/// logic might depend on chain id, so clients must rollback
/// these changes to v ∈ {0, 1}.
/// References:
/// * https://github.com/ethereumjs/ethereumjs-monorepo/blob/dc7169c16df6d36adeb6e234fcc66eb6cfc5ea3f/packages/util/src/signature.ts#L31-L62
/// * https://github.com/ethereum/go-ethereum/issues/19751#issuecomment-504900739
#[inline]
pub fn ecrecover(
    hash: &CryptoHash,
    [signature @ .., v]: &<Secp256k1 as Curve>::Signature,
) -> Option<<Secp256k1 as Curve>::PublicKey> {
    env::ecrecover(
        hash, signature, *v,
        // Do not accept malleabile signatures:
        // https://github.com/near/nearcore/blob/d73041cc1d1a70af4456fceefaceb1bf7f684fde/core/crypto/src/signature.rs#L448-L455
        true,
    )
}

#[derive(Debug, ThisError)]
pub enum ParseCurveError {
    #[error("invalid curve type")]
    InvalidCurveType,
    #[error("base58: {0}")]
    Base58(#[from] bs58::decode::Error),
}

pub enum CurveType {
    Ed25519,
    Secp256k1,
}

impl Curve for CurveType {
    type PublicKey = PublicKey;

    type Signature = Signature;
}

pub struct AsCurve<C: CurvePrefix>(PhantomData<C>);

impl<C: CurvePrefix, const N: usize> SerializeAs<[u8; N]> for AsCurve<C> {
    fn serialize_as<S>(source: &[u8; N], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        C::to_base58(source).serialize(serializer)
    }
}

impl<'de, C: CurvePrefix, const N: usize> DeserializeAs<'de, [u8; N]> for AsCurve<C> {
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
