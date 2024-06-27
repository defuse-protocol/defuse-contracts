use near_sdk::{ext_contract, Promise};

use super::IntentId;

#[ext_contract(ext_lost_found)]
pub trait LostFound {
    /// Permission-less retry failed transfer associated with given [`IntentId`].
    ///
    /// NOTE: MUST attach 1 yⓃ for security purposes.
    ///
    /// Returns `bool` indicating whether the asset was transferred successfully
    fn lost_found(&mut self, id: &IntentId) -> Promise;
}
