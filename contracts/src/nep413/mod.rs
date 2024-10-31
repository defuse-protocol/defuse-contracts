use core::fmt::Display;

use near_sdk::{borsh, env, near, CryptoHash};
use serde_with::serde_as;

pub use crate::utils::bitmap::U256;
use crate::{
    crypto::Payload,
    utils::{serde::base64::Base64, UnwrapOrPanicError},
};

#[derive(Debug, Clone, Default)]
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
pub struct Nep413Payload {
    pub message: String,

    #[serde_as(as = "Base64")]
    #[cfg_attr(
        all(feature = "abi", not(target_arch = "wasm32")),
        schemars(example = "self::examples::nonce")
    )]
    pub nonce: U256,

    pub recipient: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,
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

    pub fn with_nonce(mut self, nonce: U256) -> Self {
        self.nonce = nonce;
        self
    }

    pub fn with_recipient<S>(mut self, recipient: S) -> Self
    where
        S: Display,
    {
        self.recipient = recipient.to_string();
        self
    }

    pub fn with_callback_url(mut self, callback_url: String) -> Self {
        self.callback_url = Some(callback_url);
        self
    }
}

impl Payload for Nep413Payload {
    /// Returns SHA-256 hash of serialized payload according to
    /// [NEP-413](https://github.com/near/NEPs/blob/master/neps/nep-0413.md#signature)
    #[inline]
    fn hash(&self) -> CryptoHash {
        const NEP_NUMBER: u32 = 413;
        /// [NEP-461](https://github.com/near/NEPs/pull/461) prefix_tag
        const PREFIX_TAG: u32 = (1u32 << 31) + NEP_NUMBER;

        env::sha256_array(&borsh::to_vec(&(PREFIX_TAG, self)).unwrap_or_panic_display())
    }
}

#[cfg(all(feature = "abi", not(target_arch = "wasm32")))]
mod examples {
    use super::*;

    use near_sdk::base64::{self, Engine};

    pub fn nonce() -> String {
        base64::engine::general_purpose::STANDARD.encode(U256::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use hex_literal::hex;
    use near_sdk::serde_json::{self, json};

    #[test]
    fn test_hash() {
        let p: Nep413Payload = serde_json::from_value(json!({
          "message": "example",
          "nonce": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAP8=", // i.e. 1
          "recipient": "example.near"
        }))
        .unwrap();

        assert_eq!(
            p.hash(),
            hex!("458584c1ca632fbc6a65d2ffaaa65ead60928e7bad742ea3d02aa232f8bcf08b")
        );
    }
}
