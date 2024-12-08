mod execute;
mod relayer;
mod simulate;
mod state;

use defuse_core::{
    engine::{Engine, StateView},
    payload::multi::MultiPayload,
    DefuseError,
};
use defuse_near_utils::UnwrapOrPanic;
use execute::ExecuteInspector;
use near_plugins::{pause, Pausable};
use near_sdk::{near, FunctionError};
use simulate::SimulateInspector;

use crate::intents::{Intents, SimulationOutput, StateOutput};

use super::{Contract, ContractExt};

#[near]
impl Intents for Contract {
    #[pause(name = "intents")]
    #[inline]
    // TODO: rename intents -> signed?
    fn execute_intents(&mut self, intents: Vec<MultiPayload>) {
        let mut insp = ExecuteInspector::default();
        let mut engine = Engine::new(self, &mut insp);
        engine.execute_signed_intents(intents).unwrap_or_panic();
        engine.finalize().unwrap_or_panic();
    }

    #[pause(name = "intents")]
    #[inline]
    fn simulate_intents(&self, intents: Vec<MultiPayload>) -> SimulationOutput {
        let mut inspector = SimulateInspector::default();
        let mut engine = Engine::new(self.cached(), &mut inspector);
        engine.execute_signed_intents(intents).unwrap_or_panic();

        let unmatched_deltas = match engine.finalize() {
            Ok(_) => None,
            Err(DefuseError::UnmatchedDeltas(v)) => v,
            Err(err) => err.panic(),
        };
        SimulationOutput {
            intents_executed: inspector.intents_executed,
            min_deadline: inspector.min_deadline,
            unmatched_deltas,
            state: StateOutput { fee: self.fee() },
        }
    }
}
