use defuse_core::{
    accounts::AccountEvent,
    fees::Pips,
    intents::{token_diff::TokenDeltas, IntentExecutedEvent},
    payload::multi::MultiPayload,
    Deadline,
};

use near_plugins::AccessControllable;
use near_sdk::{ext_contract, near, Promise, PublicKey};

use crate::fees::FeesManager;

#[ext_contract(ext_intents)]
pub trait Intents: FeesManager {
    fn execute_intents(&mut self, intents: Vec<MultiPayload>);

    fn simulate_intents(&self, intents: Vec<MultiPayload>) -> SimulationOutput;
}

#[near(serializers = [json])]
#[derive(Debug, Clone)]
pub struct SimulationOutput {
    /// Intent hashes along with corresponding signers
    pub intents_executed: Vec<AccountEvent<'static, IntentExecutedEvent>>,

    /// Minimum deadline among all simulated intents
    pub min_deadline: Deadline,

    /// `token_diff`-related output
    pub token_diff: TokenDiffOutput,

    /// Additional info about current state
    pub state: StateOutput,
}

#[near(serializers = [json])]
#[derive(Debug, Clone)]
pub struct TokenDiffOutput {
    /// Closure for a **single** `token_diff` intent that needs
    /// to be added to sequence of simulated intents to keep
    /// the invariant.  
    /// Empty closure means that the invariant was not violated.
    pub closure: TokenDeltas,
}

#[near(serializers = [json])]
#[derive(Debug, Clone)]
pub struct StateOutput {
    pub fee: Pips,
}

#[ext_contract(ext_relayer_keys)]
pub trait RelayerKeys: AccessControllable {
    /// Adds access key for calling `execute_signed_intents`
    /// with allowance passed as attached deposit via `#[payable]`
    /// NOTE: requires 1yN for security purposes
    fn add_relayer_key(&mut self, public_key: PublicKey) -> Promise;

    fn do_add_relayer_key(&mut self, public_key: PublicKey);

    /// NOTE: requires 1yN for security purposes
    fn delete_relayer_key(&mut self, public_key: PublicKey) -> Promise;
}
