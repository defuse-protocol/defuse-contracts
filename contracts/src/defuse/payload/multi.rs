use derive_more::derive::From;
use near_sdk::{near, serde::de::DeserializeOwned, serde_json, CryptoHash};

use crate::{crypto::Payload, erc191, nep413::Nep413Payload};

use super::DefusePayload;

#[near(serializers = [borsh, json])]
#[serde(tag = "standard", content = "payload", rename_all = "snake_case")]
#[derive(Debug, Clone, From)]
pub enum MultiStandardPayload {
    /// See [NEP-413](https://github.com/near/NEPs/blob/master/neps/nep-0413.md)
    Nep413(Nep413Payload),

    /// See [ERC-191](https://github.com/ethereum/ercs/blob/master/ERCS/erc-191.md),
    /// [personal_sign](https://docs.metamask.io/wallet/reference/json-rpc-methods/personal_sign)
    #[from(ignore)]
    Erc191(String),
}

impl Payload for MultiStandardPayload {
    #[inline]
    fn hash(&self) -> CryptoHash {
        match self {
            Self::Nep413(payload) => payload.hash(),
            Self::Erc191(payload) => erc191::sign_hash(payload.as_bytes()),
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
            MultiStandardPayload::Nep413(payload) => payload.try_into(),
            MultiStandardPayload::Erc191(payload) => serde_json::from_str(&payload),
        }
    }
}
