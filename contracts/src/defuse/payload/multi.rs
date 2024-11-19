use derive_more::derive::From;
use near_sdk::{near, serde::de::DeserializeOwned, serde_json, CryptoHash};

use crate::{
    crypto::{CurveType, Payload, PublicKey, SignedPayload},
    solana,
};

use super::{erc191::Erc191SignedPayload, nep413::Nep413SignedPayload, DefusePayload};

#[near(serializers = [borsh, json])]
#[serde(tag = "standard", rename_all = "snake_case")]
#[derive(Debug, Clone, From)]
pub enum MultiStandardPayload {
    /// See [NEP-413](https://github.com/near/NEPs/blob/master/neps/nep-0413.md)
    Nep413(Nep413SignedPayload),

    /// See [ERC-191](https://github.com/ethereum/ercs/blob/master/ERCS/erc-191.md),
    /// [personal_sign](https://docs.metamask.io/wallet/reference/json-rpc-methods/personal_sign)
    #[from(ignore)]
    Erc191(Erc191SignedPayload),
    Solana(solana::SignedOffchainMessage),
}

impl Payload for MultiStandardPayload {
    #[inline]
    fn hash(&self) -> CryptoHash {
        match self {
            Self::Nep413(payload) => payload.hash(),
            Self::Erc191(payload) => payload.hash(),
            Self::Solana(payload) => payload.hash(),
        }
    }
}

impl SignedPayload for MultiStandardPayload {
    type Payload = MultiStandardPayload;
    type Curve = CurveType;

    fn verify(&self) -> Option<<Self::Curve as crate::crypto::Curve>::PublicKey> {
        match self {
            Self::Nep413(payload) => payload.verify().map(PublicKey::Ed25519),
            Self::Erc191(payload) => payload.verify().map(PublicKey::Secp256k1),
            Self::Solana(payload) => payload.verify().map(PublicKey::Ed25519),
        }
    }
}

impl<T> TryFrom<MultiStandardPayload> for DefusePayload<T>
where
    T: DeserializeOwned,
{
    type Error = serde_json::Error;

    fn try_from(value: MultiStandardPayload) -> Result<Self, Self::Error> {
        match value {
            MultiStandardPayload::Nep413(payload) => payload.payload.try_into(),
            MultiStandardPayload::Erc191(payload) => payload.payload.try_into(),
            MultiStandardPayload::Solana(payload) => payload.try_into(),
        }
    }
}
