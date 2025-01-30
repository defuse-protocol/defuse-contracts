use defuse_crypto::{Payload, PublicKey, SignedPayload};
use defuse_erc191::SignedErc191Payload;
use defuse_nep413::SignedNep413Payload;
use defuse_webauthn::SignedWebAuthnPayload;
use derive_more::derive::From;
use near_sdk::{near, serde::de::DeserializeOwned, serde_json, CryptoHash};

use super::{raw::SignedRawEd25519Payload, DefusePayload, ExtractDefusePayload};

#[near(serializers = [borsh, json])]
#[serde(tag = "standard", rename_all = "snake_case")]
#[derive(Debug, Clone, From)]
pub enum MultiPayload {
    Nep413(SignedNep413Payload),
    Erc191(SignedErc191Payload),
    RawEd25519(SignedRawEd25519Payload),
    WebAuthn(SignedWebAuthnPayload),
}

impl Payload for MultiPayload {
    #[inline]
    fn hash(&self) -> CryptoHash {
        match self {
            Self::Nep413(payload) => payload.hash(),
            Self::Erc191(payload) => payload.hash(),
            Self::RawEd25519(payload) => payload.hash(),
            Self::WebAuthn(payload) => payload.hash(),
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
            Self::RawEd25519(payload) => payload.verify().map(PublicKey::Ed25519),
            // TODO: allow for different curves
            Self::WebAuthn(payload) => payload.verify().map(PublicKey::P256),
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
            Self::RawEd25519(payload) => payload.extract_defuse_payload(),
            Self::WebAuthn(payload) => payload.extract_defuse_payload(),
        }
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::{bs58, AccountId};

    use super::*;

    #[test]
    fn test_raw_ed25519() {
        let p: MultiPayload = serde_json::from_str(r#"{"standard":"raw_ed25519","payload":"{\"signer_id\":\"74affa71ab030d400fdfa1bed033dfa6fd3ae34f92d17c046ebe368e80d53751\",\"verifying_contract\":\"intents.near\",\"deadline\":{\"timestamp\":1732035219},\"nonce\":\"XVoKfmScb3G+XqH9ke/fSlJ/3xO59sNhCxhpG821BH8=\",\"intents\":[{\"intent\":\"token_diff\",\"diff\":{\"nep141:base-0x833589fcd6edb6e08f4c7c32d4f71b54bda02913.omft.near\":\"-1000\",\"nep141:eth-0xdac17f958d2ee523a2206206994597c13d831ec7.omft.near\":\"998\"}}]}","public_key":"ed25519:8rVvtHWFr8hasdQGGD5WiQBTyr4iH2ruEPPVfj491RPN","signature":"ed25519:3vtbNQJHZfuV1s5DykzyjkbNLc583hnkrhTz57eDhd966iqzkor6Twgr4Loh2C195SCSEsiGfrd6KcxpjNq9ZbVj"}"#).unwrap();
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

    #[test]
    fn test_passkey() {
        let p: MultiPayload = serde_json::from_str(r##"{
  "standard": "web_authn",
  "payload": "{\"signer_id\":\"29d2fa7c2f4e0999e6a7b30ce3f84de8a5e29df0.p256\",\"verifying_contract\":\"intents.near\",\"deadline\":{\"timestamp\":1732035219},\"nonce\":\"XVoKfmScb3G+XqH9ke/fSlJ/3xO59sNhCxhpG821BH8=\",\"intents\":[{\"intent\":\"token_diff\",\"diff\":{\"nep141:base-0x833589fcd6edb6e08f4c7c32d4f71b54bda02913.omft.near\":\"-1000\",\"nep141:eth-0xdac17f958d2ee523a2206206994597c13d831ec7.omft.near\":\"998\"}}]}",
  "public_key": "p256:qE7uvcm3vHUfw6WA86giC2BRpvQtmoEK1e7aUPLPW1pfFGK4sy1mS3StbRXDx1Y71E369JRGzaJJHsCddUiTAF2",
  "signature": "p256:WkrBdaGF6BaGz6Qvtet4SdH2hCqfWDXMEAqzjwdZ5izAwRTAEefpQs8L9D1QXXPNsMSdrHTDcNeEE3GvjZke6ug",
  "client_data_json": "{\"type\":\"webauthn.get\",\"challenge\":\"63O3uH8qhqu4Akz_pQ81fF_PXERQl7g1OHLq9oGG9wE\",\"origin\":\"http://localhost:3000\"}",
  "authenticator_data": "SZYN5YgOjGh0NBcPZHZgW4_krrmihjLHmVzzuoMdl2MdAAAAAA=="
}"##).unwrap();

        let pk = p.verify().unwrap();
        assert_eq!(
            pk,
            "p256:qE7uvcm3vHUfw6WA86giC2BRpvQtmoEK1e7aUPLPW1pfFGK4sy1mS3StbRXDx1Y71E369JRGzaJJHsCddUiTAF2"
                .parse()
                .unwrap()
        );
        assert_eq!(
            pk.to_implicit_account_id(),
            "29d2fa7c2f4e0999e6a7b30ce3f84de8a5e29df0.p256"
                .parse::<AccountId>()
                .unwrap()
        )
    }
}
