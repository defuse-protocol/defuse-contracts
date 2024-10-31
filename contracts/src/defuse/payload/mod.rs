pub mod multi;
pub mod nep413;

use impl_tools::autoimpl;
use near_sdk::{near, AccountId};
use serde_with::serde_as;

use crate::{
    nep413::U256,
    utils::{serde::base64::Base64, Deadline},
};

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
#[derive(Debug, Clone)]
pub struct DefusePayload<T> {
    pub signer_id: AccountId,
    pub verifying_contract: AccountId,
    pub deadline: Deadline,
    #[serde_as(as = "Base64")]
    #[cfg_attr(
        all(feature = "abi", not(target_arch = "wasm32")),
        schemars(example = "self::examples::nonce")
    )]
    pub nonce: U256,

    #[serde(flatten)]
    pub message: T,
}

#[cfg(all(feature = "abi", not(target_arch = "wasm32")))]
mod examples {
    use super::*;

    use near_sdk::base64::{self, Engine};

    pub fn nonce() -> String {
        base64::engine::general_purpose::STANDARD.encode(U256::default())
    }
}
