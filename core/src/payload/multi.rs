use defuse_crypto::{Payload, PublicKey, SignedPayload};
use defuse_erc191::SignedErc191Payload;
use defuse_nep413::SignedNep413Payload;
use derive_more::derive::From;
use near_sdk::{near, serde::de::DeserializeOwned, serde_json, CryptoHash};

use super::{DefusePayload, ExtractDefusePayload};

#[near(serializers = [borsh, json])]
#[serde(tag = "standard", rename_all = "snake_case")]
#[derive(Debug, Clone, From)]
pub enum MultiPayload {
    Nep413(SignedNep413Payload),
    Erc191(SignedErc191Payload),
    // TODO: Solana
}

impl Payload for MultiPayload {
    #[inline]
    fn hash(&self) -> CryptoHash {
        match self {
            Self::Nep413(payload) => payload.hash(),
            Self::Erc191(payload) => payload.hash(),
        }
    }
}

impl SignedPayload for MultiPayload {
    type PublicKey = PublicKey;

    #[inline]
    fn verify(&self) -> Option<Self::PublicKey> {
        match self {
            Self::Nep413(payload) => payload.verify().map(PublicKey::Ed25519),
            Self::Erc191(payload) => payload.verify().map(PublicKey::Secp256k1),
        }
    }
}

impl<T> ExtractDefusePayload<T> for MultiPayload
where
    T: DeserializeOwned,
{
    type Error = serde_json::Error;

    #[inline]
    fn extract_defuse_payload(self) -> Result<DefusePayload<T>, Self::Error> {
        match self {
            Self::Nep413(payload) => payload.extract_defuse_payload(),
            Self::Erc191(payload) => payload.extract_defuse_payload(),
        }
    }
}
