use derive_more::derive::From;
use near_sdk::{near, serde::de::DeserializeOwned, serde_json, CryptoHash};

use crate::{
    crypto::{Payload, PublicKey, SignedPayload},
    erc191::SignedErc191Payload,
    nep413::SignedNep413Payload,
};

use super::{raw::SignedRawEd25519Payload, DefusePayload};

#[near(serializers = [borsh, json])]
#[serde(tag = "standard", rename_all = "snake_case")]
#[derive(Debug, Clone, From)]
pub enum MultiStandardPayload {
    /// See [NEP-413](https://github.com/near/NEPs/blob/master/neps/nep-0413.md)
    Nep413(SignedNep413Payload),

    /// See [ERC-191](https://github.com/ethereum/ercs/blob/master/ERCS/erc-191.md),
    /// [personal_sign](https://docs.metamask.io/wallet/reference/json-rpc-methods/personal_sign)
    #[from(ignore)]
    Erc191(SignedErc191Payload),
    RawEd25519(SignedRawEd25519Payload),
}

impl Payload for MultiStandardPayload {
    #[inline]
    fn hash(&self) -> CryptoHash {
        match self {
            Self::Nep413(payload) => payload.hash(),
            Self::Erc191(payload) => payload.hash(),
            Self::RawEd25519(payload) => payload.hash(),
        }
    }
}

impl SignedPayload for MultiStandardPayload {
    type PublicKey = PublicKey;

    #[inline]
    fn verify(&self) -> Option<Self::PublicKey> {
        match self {
            MultiStandardPayload::Nep413(payload) => payload.verify().map(PublicKey::Ed25519),
            MultiStandardPayload::Erc191(payload) => payload.verify().map(PublicKey::Secp256k1),
            MultiStandardPayload::RawEd25519(payload) => payload.verify().map(PublicKey::Ed25519),
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
            MultiStandardPayload::Erc191(payload) => serde_json::from_str(&payload.payload),
            MultiStandardPayload::RawEd25519(payload) => payload.try_into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::bs58;

    use super::*;

    #[test]
    fn test_json() {
        let p: MultiStandardPayload = serde_json::from_str(r#"{"standard":"raw_ed25519","payload":"{\"signer_id\":\"74affa71ab030d400fdfa1bed033dfa6fd3ae34f92d17c046ebe368e80d53751\",\"verifying_contract\":\"intents.near\",\"deadline\":{\"timestamp\":1732035219},\"nonce\":\"XVoKfmScb3G+XqH9ke/fSlJ/3xO59sNhCxhpG821BH8=\",\"intents\":[{\"intent\":\"token_diff\",\"diff\":{\"nep141:base-0x833589fcd6edb6e08f4c7c32d4f71b54bda02913.omft.near\":\"-1000\",\"nep141:eth-0xdac17f958d2ee523a2206206994597c13d831ec7.omft.near\":\"998\"}}]}","public_key":"ed25519:8rVvtHWFr8hasdQGGD5WiQBTyr4iH2ruEPPVfj491RPN","signature":"ed25519:3vtbNQJHZfuV1s5DykzyjkbNLc583hnkrhTz57eDhd966iqzkor6Twgr4Loh2C195SCSEsiGfrd6KcxpjNq9ZbVj"}"#).unwrap();
        assert_eq!(
            bs58::encode(p.hash()).into_string(),
            "8LKE47o44ybZQR9ozLyDnvMDTh4Ao5ipy2mJWsYByG5Q"
        );
        assert_eq!(
            p.verify().unwrap(),
            "ed25519:8rVvtHWFr8hasdQGGD5WiQBTyr4iH2ruEPPVfj491RPN"
                .parse()
                .unwrap()
        );
    }
}
