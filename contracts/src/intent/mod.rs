mod error;
mod intent;

pub use self::{error::*, intent::*};

use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::{ext_contract, AccountId, Promise};

#[ext_contract(ext_intent_contract)]
pub trait IntentContract: FungibleTokenReceiver {
    /// Return pending intent by id.
    fn get_intent(&self, id: &String) -> Option<&Intent>;

    /// Rollback created intent and refund tokens to the intent's initiator.
    /// The transaction could be called by an intent initiator or owner.
    ///
    /// # Panics
    ///
    /// The panic occurs if intent doesn't exist of caller is not allowed.
    fn rollback_intent(&mut self, id: &String) -> Promise;

    /// Add a new solver to the whitelist.
    fn add_solver(&mut self, solver_id: AccountId);

    /// Check if the provided solver is allowed.
    fn is_allowed_solver(&self, solver_id: AccountId) -> bool;
}
