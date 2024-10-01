use core::fmt::Display;

use impl_tools::autoimpl;
use near_sdk::{
    borsh::{self, BorshSerialize},
    env::sha256_array,
    near,
};
use serde_with::{serde_as, DisplayFromStr};

use crate::{
    crypto::Payload,
    utils::{integer::U256, UnwrapOrPanic},
};

pub type Nonce = U256;

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
pub struct Nep413Payload<T = String> {
    #[borsh(
        serialize_with = "crate::utils::borsh::as_base64",
        deserialize_with = "crate::utils::borsh::from_base64"
    )]
    pub message: T,

    #[serde_as(as = "DisplayFromStr")]
    pub nonce: Nonce,

    pub recipient: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,
}

impl<T> Nep413Payload<T>
where
    T: BorshSerialize,
{
    #[inline]
    pub fn new(message: T) -> Self {
        Self {
            message,
            nonce: Default::default(),
            recipient: Default::default(),
            callback_url: Default::default(),
        }
    }

    pub fn with_nonce(mut self, nonce: Nonce) -> Self {
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
    T: BorshSerialize,
{
    /// Returns SHA-256 hash of serialized payload according to
    /// [NEP-413](https://github.com/near/NEPs/blob/master/neps/nep-0413.md#signature)
    #[inline]
    fn hash(&self) -> [u8; 32] {
        const NEP_NUMBER: u32 = 413;
        /// [NEP-461](https://github.com/near/NEPs/pull/461) prefix_tag
        const PREFIX_TAG: u32 = (1u32 << 31) + NEP_NUMBER;

        sha256_array(&borsh::to_vec(&(PREFIX_TAG, self)).unwrap_or_panic_display())
    }
}
