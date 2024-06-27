use near_sdk::{ext_contract, PromiseOrValue};

use super::IntentId;

#[ext_contract(ext_rollback)]
pub trait Rollback {
    /// Rollback the intent and asset_in to the initiator.
    ///
    /// NOTE: MUST attach 1 yⓃ for security purposes.
    fn rollback_intent(&mut self, id: &IntentId) -> PromiseOrValue<bool>;
}
