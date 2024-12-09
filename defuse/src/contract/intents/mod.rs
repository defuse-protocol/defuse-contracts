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
use defuse_nep245::MtEvent;
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
    fn execute_intents(&mut self, signed: Vec<MultiPayload>) {
        Engine::new(self, ExecuteInspector::default())
            .execute_signed_intents(signed)
            .unwrap_or_panic()
            .as_mt_event()
            .as_ref()
            .map(MtEvent::emit);
    }

    #[pause(name = "intents")]
    #[inline]
    fn simulate_intents(&self, signed: Vec<MultiPayload>) -> SimulationOutput {
        let mut inspector = SimulateInspector::default();
        let engine = Engine::new(self.cached(), &mut inspector);

        let unmatched_deltas = match engine.execute_signed_intents(signed) {
            // do not log transfers
            Ok(_) => None,
            Err(DefuseError::InvariantViolated(v)) => Some(v),
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
