use near_sdk::{ext_contract, near, AccountId, Promise};

use super::{Asset, IntentId};

#[ext_contract(ext_lost_found)]
pub trait LostFound {
    /// Permission-less retry failed transfer associated with given [`IntentId`].
    ///
    /// NOTE: MUST attach 1 yⓃ for security purposes.
    ///
    /// Returns `bool` indicating whether the asset was transferred successfully
    fn lost_found(&mut self, id: &IntentId) -> Promise;
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[near(serializers = [borsh, json])]
pub struct LostAsset {
    #[serde(flatten)]
    /// The asset that was lost while execute/rollback.
    pub asset: Asset,
    /// Where to send the lost asset
    pub recipient: AccountId,
}
