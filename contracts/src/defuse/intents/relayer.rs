use near_plugins::AccessControllable;
use near_sdk::{ext_contract, Promise, PublicKey};

use super::IntentsExecutor;

#[ext_contract(ext_relayer_keys)]
pub trait RelayerKeys: IntentsExecutor + AccessControllable {
    /// Adds access key for calling `execute_signed_intents`
    /// with allowance passed as attached deposit via `#[payable]`
    fn add_relayer_key(&mut self, public_key: PublicKey) -> Promise;

    fn do_add_relayer_key(&mut self, public_key: PublicKey);

    /// NOTE: requires 1yN for security purposes
    fn delete_relayer_key(&mut self, public_key: PublicKey) -> Promise;
}
