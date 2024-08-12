use near_sdk::{ext_contract, near, Promise};

use super::{GenericAccount, IntentId};

#[ext_contract(ext_lost_found)]
pub trait LostFound {
    /// Permission-less retry failed transfer associated with given [`IntentId`].
    ///
    /// NOTE: MUST attach 1 yⓃ for security purposes.
    ///
    /// Returns `bool` indicating whether the asset was transferred successfully.
    fn lost_found(&mut self, id: &IntentId) -> Promise;
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[near(serializers = [borsh, json])]
#[serde(rename_all = "snake_case", tag = "direction")]
pub enum LostAsset {
    /// Failed to transfer [`asset_in`](super::SwapIntent::asset_in).
    AssetIn {
        /// Where `asset_in` was meant to be sent.
        recipient: GenericAccount,
    },
    /// Failed to transfer [`asset_out`](super::SwapIntent::asset_out)
    /// to its recipient.
    AssetOut,
}
