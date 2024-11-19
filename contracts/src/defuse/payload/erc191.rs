use impl_tools::autoimpl;
use near_sdk::{
    near,
    serde::{de::DeserializeOwned, Deserialize},
    serde_json, CryptoHash,
};
use serde_with::serde_as;

use crate::{
    crypto::{ecrecover, AsCurve, Curve, Payload, Secp256k1, SignedPayload},
    erc191::Erc191Payload,
};

use super::DefusePayload;

#[serde_as]
#[near(serializers = [borsh, json])]
#[autoimpl(Deref using self.payload)]
#[derive(Debug, Clone)]
pub struct Erc191SignedPayload {
    pub payload: Erc191Payload<'static>,

    #[serde_as(as = "AsCurve<Secp256k1>")]
    pub signature: <Secp256k1 as Curve>::Signature,
}

impl SignedPayload for Erc191SignedPayload {
    type Payload = Erc191Payload<'static>;
    type Curve = Secp256k1;

    #[inline]
    fn verify(&self) -> Option<<Self::Curve as Curve>::PublicKey> {
        ecrecover(&self.hash(), &self.signature)
    }
}

impl<'a, T> TryFrom<Erc191Payload<'static>> for DefusePayload<T>
where
    T: DeserializeOwned,
{
    type Error = serde_json::Error;

    #[inline]
    fn try_from(value: Erc191Payload<'static>) -> Result<Self, Self::Error> {
        serde_json::from_str(&value.0)
    }
}
