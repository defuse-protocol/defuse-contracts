use near_sdk::{ext_contract, PromiseOrValue};

use super::SwapIntentAction;

#[ext_contract(ext_native_action)]
pub trait NativeAction {
    /// MUST be `#[payable]`.
    /// Returns true if the action succeeded.
    fn native_action(&mut self, action: SwapIntentAction) -> PromiseOrValue<bool>;
}
