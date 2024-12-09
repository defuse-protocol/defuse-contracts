use core::fmt::Display;

use defuse_crypto::{serde::AsCurve, CryptoHash, Curve, Ed25519, Payload, SignedPayload};
use defuse_near_utils::UnwrapOrPanicError;
use defuse_nep461::{OffchainMessage, SignedMessageNep};
use defuse_serde_utils::base64::Base64;
use impl_tools::autoimpl;
use near_sdk::{borsh, env, near};
use serde_with::serde_as;

/// See [NEP-413](https://github.com/near/NEPs/blob/master/neps/nep-0413.md)
#[cfg_attr(
    all(feature = "abi", not(target_arch = "wasm32")),
    serde_as(schemars = true)
)]
#[cfg_attr(
    not(all(feature = "abi", not(target_arch = "wasm32"))),
    serde_as(schemars = false)
)]
#[near(serializers = [borsh, json])]
#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone)]
pub struct Nep413Payload {
    pub message: String,

    #[serde_as(as = "Base64")]
    pub nonce: [u8; 32],

    pub recipient: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,
}

impl SignedMessageNep for Nep413Payload {
    const NEP_NUMBER: u32 = 413;
}

impl Nep413Payload {
    #[inline]
    pub fn new(message: String) -> Self {
        Self {
            message,
            nonce: Default::default(),
            recipient: Default::default(),
            callback_url: Default::default(),
        }
    }

    #[inline]
    pub fn with_nonce(mut self, nonce: [u8; 32]) -> Self {
        self.nonce = nonce;
        self
    }

    #[inline]
    pub fn with_recipient<S>(mut self, recipient: S) -> Self
    where
        S: Display,
    {
        self.recipient = recipient.to_string();
        self
    }

    #[inline]
    pub fn with_callback_url(mut self, callback_url: String) -> Self {
        self.callback_url = Some(callback_url);
        self
    }

    #[inline]
    pub fn prehash(&self) -> Vec<u8> {
        borsh::to_vec(&(Self::OFFCHAIN_PREFIX_TAG, self)).unwrap_or_panic_display()
    }
}

impl Payload for Nep413Payload {
    #[inline]
    fn hash(&self) -> CryptoHash {
        env::sha256_array(&self.prehash())
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
#[near(serializers = [borsh, json])]
#[autoimpl(Deref using self.payload)]
#[derive(Debug, Clone)]
pub struct SignedNep413Payload {
    pub payload: Nep413Payload,

    #[serde_as(as = "AsCurve<Ed25519>")]
    pub public_key: <Ed25519 as Curve>::PublicKey,
    #[serde_as(as = "AsCurve<Ed25519>")]
    pub signature: <Ed25519 as Curve>::Signature,
}

impl Payload for SignedNep413Payload {
    #[inline]
    fn hash(&self) -> CryptoHash {
        self.payload.hash()
    }
}

impl SignedPayload for SignedNep413Payload {
    type PublicKey = <Ed25519 as Curve>::PublicKey;

    #[inline]
    fn verify(&self) -> Option<Self::PublicKey> {
        env::ed25519_verify(&self.signature, &self.hash(), &self.public_key)
            .then_some(&self.public_key)
            .cloned()
    }
}
