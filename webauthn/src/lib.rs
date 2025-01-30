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
    #[serde_as(as = "Base64<UrlSafe, Unpadded>")]
    pub authenticator_data: Vec<u8>,
    pub client_data_json: String,
    // TODO: allow for more algorithms
    #[serde_as(as = "AsCurve<P256>")]
    pub public_key: <P256 as Curve>::PublicKey,
    #[serde_as(as = "AsCurve<P256>")]
    pub signature: <P256 as Curve>::Signature,
}

impl SignedWebAuthnPayload {
    const AUTH_DATA_FLAGS_UP: u8 = 0 << 0;
    const AUTH_DATA_FLAGS_UV: u8 = 0 << 2;
    const AUTH_DATA_FLAGS_BE: u8 = 0 << 3;
    const AUTH_DATA_FLAGS_BS: u8 = 0 << 4;

    /// https://w3c.github.io/webauthn/#sctn-verifying-assertion
    fn verify_flags(flags: u8, require_user_verification: bool) -> bool {
        // TODO: veryfy other fields from auth_data?

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
        // TODO: ENVELOPE: prefix hash with version?
        env::sha256_array(self.payload.as_bytes())
    }
}

impl SignedPayload for SignedWebAuthnPayload {
    type PublicKey = <P256 as Curve>::PublicKey;

    /// https://w3c.github.io/webauthn/#sctn-verifying-assertion
    fn verify(&self) -> Option<Self::PublicKey> {
        // verify authData flags first
        if self.authenticator_data.len() < 37
            || !Self::verify_flags(self.authenticator_data[32], false)
        {
            return None;
        }

        let collected_client_data: CollectedClientData =
            serde_json::from_str(&self.client_data_json).ok()?;

        // 10. Verify that the value of C.type is the string webauthn.get.
        if collected_client_data.typ != ClientDataType::Get {
            return None;
        }

        // 11. Verify that the value of C.challenge equals the base64url
        // encoding of pkOptions.challenge
        if collected_client_data.challenge != self.hash() {
            return None;
        }

        // // TODO: origin & cross-origin?

        // 20. Let hash be the result of computing a hash over the cData using
        // SHA-256
        let hash = env::sha256_array(self.client_data_json.as_bytes());

        // 21. Using credentialRecord.publicKey, verify that sig is a valid signature
        // over the binary concatenation of authData and hash.

        let prehashed =
            env::sha256_array(&[self.authenticator_data.as_slice(), hash.as_slice()].concat());
        P256::verify(&self.signature, &prehashed, &self.public_key)
    }
}

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
