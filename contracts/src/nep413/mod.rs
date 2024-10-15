use core::{fmt::Display, str::FromStr};

use impl_tools::autoimpl;
use near_sdk::{borsh, env::sha256_array, near, CryptoHash};
use serde_with::{serde_as, DisplayFromStr};

pub use crate::utils::bitmap::U256;
use crate::{
    crypto::Payload,
    utils::{serde::base64::Base64, UnwrapOrPanic},
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
#[autoimpl(Deref using self.message)]
#[autoimpl(DerefMut using self.message)]
pub struct Nep413Payload<T = String> {
    #[borsh(
        bound(serialize = "T: Display", deserialize = "T: FromStr<Err: Display>"),
        serialize_with = "crate::utils::borsh::DisplayFromStr::serialize",
        deserialize_with = "crate::utils::borsh::DisplayFromStr::deserialize"
    )]
    #[serde_as(as = "DisplayFromStr")]
    #[serde(bound(serialize = "T: Display", deserialize = "T: FromStr<Err: Display>"))]
    pub message: T,

    #[serde_as(as = "Base64")]
    pub nonce: U256,

    pub recipient: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,
}

impl<T> Nep413Payload<T> {
    #[inline]
    pub fn new(message: T) -> Self {
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

impl<T> Payload for Nep413Payload<T>
where
    T: Display,
{
    /// Returns SHA-256 hash of serialized payload according to
    /// [NEP-413](https://github.com/near/NEPs/blob/master/neps/nep-0413.md#signature)
    #[inline]
    fn hash(&self) -> CryptoHash {
        const NEP_NUMBER: u32 = 413;
        /// [NEP-461](https://github.com/near/NEPs/pull/461) prefix_tag
        const PREFIX_TAG: u32 = (1u32 << 31) + NEP_NUMBER;

        sha256_array(&borsh::to_vec(&(PREFIX_TAG, self)).unwrap_or_panic_display())
    }
}
