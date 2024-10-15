use near_sdk::near;
use serde_with::serde_as;

use crate::{crypto::PublicKey, nep413::U256, utils::serde::base64::Base64};

/// Add public key to the signer account
#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct AddPublicKey {
    pub public_key: PublicKey,
}

/// Remove public key to the signer account
#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct RemovePublicKey {
    pub public_key: PublicKey,
}

/// Invalidate given nonces TODO: error?
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
pub struct InvalidateNonces {
    #[serde_as(as = "Vec<Base64>")]
    pub nonces: Vec<U256>,
}
