use defuse_crypto::{serde::AsCurve, Curve, Payload, SignedPayload, P256};
use defuse_serde_utils::base64::{Base64, Unpadded, UrlSafe};
use near_sdk::{env, near, serde_json, CryptoHash};
use serde_with::serde_as;

#[cfg_attr(
    all(feature = "abi", not(target_arch = "wasm32")),
    serde_as(schemars = true)
)]
#[cfg_attr(
    not(all(feature = "abi", not(target_arch = "wasm32"))),
    serde_as(schemars = false)
)]
#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct SignedWebAuthnPayload {
    pub payload: String,

    /// Base64Url-encodded [authenticatorData](https://w3c.github.io/webauthn/#authenticator-data)
    #[serde_as(as = "Base64<UrlSafe, Unpadded>")]
    pub authenticator_data: Vec<u8>,
    /// Serialized [clientDataJSON](https://w3c.github.io/webauthn/#dom-authenticatorresponse-clientdatajson)
    pub client_data_json: String,

    #[serde_as(as = "AsCurve<P256>")]
    pub public_key: <P256 as Curve>::PublicKey,
    #[serde_as(as = "AsCurve<P256>")]
    pub signature: <P256 as Curve>::Signature,
}

impl SignedWebAuthnPayload {
    #[allow(clippy::identity_op)]
    const AUTH_DATA_FLAGS_UP: u8 = 1 << 0;
    const AUTH_DATA_FLAGS_UV: u8 = 1 << 2;
    const AUTH_DATA_FLAGS_BE: u8 = 1 << 3;
    const AUTH_DATA_FLAGS_BS: u8 = 1 << 4;

    /// https://w3c.github.io/webauthn/#sctn-verifying-assertion
    fn verify_flags(flags: u8, require_user_verification: bool) -> bool {
        // 16. Verify that the UP bit of the flags in authData is set.
        if flags & Self::AUTH_DATA_FLAGS_UP != Self::AUTH_DATA_FLAGS_UP {
            return false;
        }

        // 17. If user verification was determined to be required, verify that
        // the UV bit of the flags in authData is set. Otherwise, ignore the
        // value of the UV flag.
        if require_user_verification
            && (flags & Self::AUTH_DATA_FLAGS_UV != Self::AUTH_DATA_FLAGS_UV)
        {
            return false;
        }

        // 18. If the BE bit of the flags in authData is not set, verify that
        // the BS bit is not set.
        if (flags & Self::AUTH_DATA_FLAGS_BE != Self::AUTH_DATA_FLAGS_BE)
            && (flags & Self::AUTH_DATA_FLAGS_BS == Self::AUTH_DATA_FLAGS_BS)
        {
            return false;
        }

        true
    }
}

impl Payload for SignedWebAuthnPayload {
    #[inline]
    fn hash(&self) -> CryptoHash {
        env::sha256_array(self.payload.as_bytes())
    }
}

impl SignedPayload for SignedWebAuthnPayload {
    type PublicKey = <P256 as Curve>::PublicKey;

    /// https://w3c.github.io/webauthn/#sctn-verifying-assertion
    ///
    /// Credits to:
    /// * [ERC-4337 Smart Wallet](https://github.com/passkeys-4337/smart-wallet/blob/f3aa9fd44646fde0316fc810e21cc553a9ed73e0/contracts/src/WebAuthn.sol#L75-L172)
    /// * [CAP-0051](https://github.com/stellar/stellar-protocol/blob/master/core/cap-0051.md)
    fn verify(&self) -> Option<Self::PublicKey> {
        // verify authData flags
        if self.authenticator_data.len() < 37
            || !Self::verify_flags(self.authenticator_data[32], false)
        {
            return None;
        }

        // 10. Verify that the value of C.type is the string webauthn.get.
        let c: CollectedClientData = serde_json::from_str(&self.client_data_json).ok()?;
        if c.typ != ClientDataType::Get {
            return None;
        }

        // 11. Verify that the value of C.challenge equals the base64url
        // encoding of pkOptions.challenge
        //
        // In our case, challenge is a hash of the payload
        if c.challenge != self.hash() {
            return None;
        }

        // 20. Let hash be the result of computing a hash over the cData using
        // SHA-256
        let hash = env::sha256_array(self.client_data_json.as_bytes());

        // 21. Using credentialRecord.publicKey, verify that sig is a valid
        // signature over the binary concatenation of authData and hash.
        let message = [self.authenticator_data.as_slice(), hash.as_slice()].concat();
        // Only [COSE ES256 (-7) algorithm](https://www.iana.org/assignments/cose/cose.xhtml#algorithms)
        // is supported by now: P256 (a.k.a secp256r1) over SHA-256.
        // We use host impl of SHA-256 here to reduce gas consumption.
        let prehashed = env::sha256_array(&message);
        P256::verify(&self.signature, &prehashed, &self.public_key)
    }
}

/// https://w3c.github.io/webauthn/#dictdef-collectedclientdata
#[cfg_attr(
    all(feature = "abi", not(target_arch = "wasm32")),
    serde_as(schemars = true)
)]
#[cfg_attr(
    not(all(feature = "abi", not(target_arch = "wasm32"))),
    serde_as(schemars = false)
)]
#[near(serializers = [json])]
#[derive(Debug, Clone)]
pub struct CollectedClientData {
    #[serde(rename = "type")]
    pub typ: ClientDataType,

    #[serde_as(as = "Base64<UrlSafe, Unpadded>")]
    pub challenge: Vec<u8>,
}

#[near(serializers = [json])]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientDataType {
    /// Serializes to the string `"webauthn.create"`
    #[serde(rename = "webauthn.create")]
    Create,

    /// Serializes to the string `"webauthn.get"`
    #[serde(rename = "webauthn.get")]
    Get,
}

#[cfg(test)]
mod tests {
    use super::*;
    use defuse_crypto::PublicKey;
    use near_sdk::{serde_json, AccountIdRef};

    #[test]
    fn test() {
        let p: SignedWebAuthnPayload = serde_json::from_str(r#"{
  "standard": "web_authn",
  "payload": "{\"signer_id\":\"0x3602b546589a8fcafdce7fad64a46f91db0e4d50\",\"verifying_contract\":\"defuse.test.near\",\"deadline\":\"2025-03-30T00:00:00Z\",\"nonce\":\"A3nsY1GMVjzyXL3mUzOOP3KT+5a0Ruy+QDNWPhchnxM=\",\"intents\":[{\"intent\":\"transfer\",\"receiver_id\":\"user1.test.near\",\"tokens\":{\"nep141:ft1.poa-factory.test.near\":\"1000\"}}]}",
  "public_key": "p256:2V8Np9vGqLiwVZ8qmMmpkxU7CTRqje4WtwFeLimSwuuyF1rddQK5fELiMgxUnYbVjbZHCNnGc6fAe4JeDcVxgj3Q",
  "signature": "p256:3KBMZ72BHUiVfE1ey5dpi3KgbXvSEf9kuxgBEax7qLBQtidZExxxjjQk1hTTGFRrPvUoEStfrjoFNVVW4Abar94W",
  "client_data_json": "{\"type\":\"webauthn.get\",\"challenge\":\"4cveZsIe6p-WaEcL-Lhtzt3SZuXbYsjDdlFhLNrSjjk\",\"origin\":\"https://defuse-widget-git-feat-passkeys-defuse-94bbc1b2.vercel.app\"}",
  "authenticator_data": "933cQogpBzE3RSAYSAkfWoNEcBd3X84PxE8iRrRVxMgdAAAAAA=="
}"#).unwrap();

        let public_key = PublicKey::P256(p.verify().expect("invalid signature"));
        assert_eq!(
            public_key,
            "p256:2V8Np9vGqLiwVZ8qmMmpkxU7CTRqje4WtwFeLimSwuuyF1rddQK5fELiMgxUnYbVjbZHCNnGc6fAe4JeDcVxgj3Q"
                .parse()
                .unwrap(),
        );
        assert_eq!(
            public_key.to_implicit_account_id(),
            AccountIdRef::new_or_panic("0x3602b546589a8fcafdce7fad64a46f91db0e4d50")
        );
    }
}
