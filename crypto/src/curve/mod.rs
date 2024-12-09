mod ed25519;
mod secp256k1;

pub use self::{ed25519::*, secp256k1::*};

use near_sdk::bs58;
use strum::{Display, EnumString, IntoStaticStr};
use thiserror::Error as ThisError;

pub trait Curve {
    type PublicKey;
    type Signature;

    /// Message that can be signed by this curve
    type Message: AsRef<[u8]> + ?Sized;

    /// Public key that should be known prior to verification
    type VerifyingKey;

    fn verify(
        &self,
        signature: &Self::Signature,
        message: &Self::Message,
        verifying_key: &Self::VerifyingKey,
    ) -> Option<Self::PublicKey>;
}

#[derive(Display, IntoStaticStr, EnumString)]
#[strum(serialize_all = "snake_case", ascii_case_insensitive)]
pub enum CurveType {
    Ed25519,
    Secp256k1,
}

pub trait TypedCurve: Curve {
    const CURVE_TYPE: CurveType;

    #[inline]
    fn to_base58(bytes: impl AsRef<[u8]>) -> String {
        format!(
            "{}:{}",
            Self::CURVE_TYPE,
            bs58::encode(bytes.as_ref()).into_string()
        )
    }

    fn parse_base58<const N: usize>(s: impl AsRef<str>) -> Result<[u8; N], ParseCurveError> {
        let s = s.as_ref();
        let data = if let Some((curve, data)) = s.split_once(':') {
            if !curve.eq_ignore_ascii_case(Self::CURVE_TYPE.into()) {
                return Err(ParseCurveError::WrongCurveType);
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

#[derive(Debug, ThisError)]
pub enum ParseCurveError {
    #[error("wrong curve type")]
    WrongCurveType,
    #[error("base58: {0}")]
    Base58(#[from] bs58::decode::Error),
}
