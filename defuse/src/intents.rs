use std::collections::BTreeMap;

use defuse_core::{
    accounts::AccountEvent,
    fees::Pips,
    intents::{token_diff::TokenDeltas, IntentEvent},
    payload::multi::MultiPayload,
    tokens::TokenAmounts,
    Deadline,
};

use near_plugins::AccessControllable;
use near_sdk::{ext_contract, near, Promise, PublicKey};
use serde_with::{serde_as, DisplayFromStr};

use crate::fees::FeesManager;

#[ext_contract(ext_intents)]
pub trait Intents: FeesManager {
    fn execute_intents(&mut self, intents: Vec<MultiPayload>);

    fn simulate_intents(&self, intents: Vec<MultiPayload>) -> SimulationOutput;
}

#[cfg_attr(
    all(feature = "abi", not(target_arch = "wasm32")),
    serde_as(schemars = true)
)]
#[cfg_attr(
    not(all(feature = "abi", not(target_arch = "wasm32"))),
    serde_as(schemars = false)
)]
#[near(serializers = [json])]
#[derive(Debug, Clone)]
pub struct SimulationOutput {
    /// Intent hashes along with corresponding signers
    pub intents_executed: Vec<IntentEvent<AccountEvent<'static, ()>>>,

    /// Minimum deadline among all simulated intents
    pub min_deadline: Deadline,

    /// Unmatched token deltas needed to keep the invariant.
    /// If not empty, can be used along with fee to calculate `token_diff` closure.
    #[serde_as(as = "Option<TokenAmounts<BTreeMap<_, DisplayFromStr>>>")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unmatched_deltas: Option<TokenDeltas>,

    /// Additional info about current state
    pub state: StateOutput,
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
