use impl_tools::autoimpl;
use near_sdk::{
    env, near,
    serde::de::{self, DeserializeOwned},
    serde_json, AccountId, CryptoHash,
};
use serde_with::serde_as;

use crate::{
    crypto::{AsCurve, Curve, Ed25519, Payload, SignedPayload},
    nep413::Nep413Payload,
    utils::Deadline,
};

use super::DefusePayload;

#[serde_as]
#[near(serializers = [borsh, json])]
#[autoimpl(Deref using self.payload)]
#[derive(Debug, Clone)]
pub struct Nep413SignedPayload {
    pub payload: Nep413Payload,

    #[serde_as(as = "AsCurve<Ed25519>")]
    pub public_key: <Ed25519 as Curve>::PublicKey,
    #[serde_as(as = "AsCurve<Ed25519>")]
    pub signature: <Ed25519 as Curve>::Signature,
}

impl Payload for Nep413SignedPayload {
    #[inline]
    fn hash(&self) -> CryptoHash {
        self.payload.hash()
    }
}

impl SignedPayload for Nep413SignedPayload {
    type Payload = Nep413Payload;
    type Curve = Ed25519;

    #[inline]
    fn verify(&self) -> Option<<Self::Curve as Curve>::PublicKey> {
        env::ed25519_verify(&self.signature, &self.hash(), &self.public_key)
            .then_some(&self.public_key)
            .cloned()
    }
}

impl<T> TryFrom<Nep413SignedPayload> for DefusePayload<T>
where
    T: DeserializeOwned,
{
    type Error = serde_json::Error;

    #[inline]
    fn try_from(value: Nep413SignedPayload) -> Result<Self, Self::Error> {
        value.payload.try_into()
    }
}

#[near(serializers = [json])]
#[autoimpl(Deref using self.message)]
#[autoimpl(DerefMut using self.message)]
#[derive(Debug, Clone)]
pub struct Nep413DefuseMessage<T> {
    pub signer_id: AccountId,

    pub deadline: Deadline,

    #[serde(flatten)]
    pub message: T,
}

impl<T> TryFrom<Nep413Payload> for DefusePayload<T>
where
    T: DeserializeOwned,
{
    type Error = serde_json::Error;

    fn try_from(
        Nep413Payload {
            message,
            nonce,
            recipient,
            callback_url: _,
        }: Nep413Payload,
    ) -> Result<Self, Self::Error> {
        let Nep413DefuseMessage {
            signer_id,
            deadline,
            message,
        } = serde_json::from_str(&message)?;
        Ok(DefusePayload {
            signer_id,
            verifying_contract: recipient.parse().map_err(|_| {
                de::Error::invalid_value(de::Unexpected::Str(&recipient), &"AccountId")
            })?,
            deadline,
            nonce,
            message,
        })
    }
}
