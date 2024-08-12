use near_sdk::{ext_contract, PromiseOrValue};

#[ext_contract(ext_native_action)]
pub trait NativeReceiver {
    /// MUST be `#[payable]`.
    /// Returns true if the action succeeded.
    fn native_on_transfer(&mut self, msg: String) -> PromiseOrValue<bool>;
}
