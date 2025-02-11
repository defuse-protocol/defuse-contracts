use near_sdk::env;

use super::{Curve, CurveType, TypedCurve};

pub struct Ed25519;

impl Curve for Ed25519 {
    type PublicKey = [u8; 32];
    type Signature = [u8; 64];

    type Message = [u8];
    type VerifyingKey = Self::PublicKey;

    #[inline]
    fn verify(
        signature: &Self::Signature,
        message: &Self::Message,
        public_key: &Self::VerifyingKey,
    ) -> Option<Self::PublicKey> {
        env::ed25519_verify(signature, message, public_key)
            .then_some(public_key)
            .copied()
    }
}

impl TypedCurve for Ed25519 {
    const CURVE_TYPE: CurveType = CurveType::Ed25519;
}
