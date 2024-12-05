use defuse_core::{
    accounts::AccountEvent,
    engine::Inspector,
    intents::{token_diff::TokenDiff, IntentExecutedEvent},
    tokens::TokenId,
    Deadline,
};
use near_sdk::{AccountIdRef, CryptoHash};

pub struct SimulateInspector {
    pub intents_executed: Vec<AccountEvent<'static, IntentExecutedEvent>>,
    pub min_deadline: Deadline,
}

impl Default for SimulateInspector {
    fn default() -> Self {
        Self {
            intents_executed: Default::default(),
            min_deadline: Deadline::MAX,
        }
    }
}

impl Inspector for SimulateInspector {
    fn on_deadline(&mut self, deadline: Deadline) {
        self.min_deadline = self.min_deadline.min(deadline);
    }

    fn on_token_diff(&mut self, owner_id: &AccountIdRef, token_diff: &TokenDiff) {
        todo!()
    }

    fn on_intent_executed(&mut self, signer_id: &AccountIdRef, hash: CryptoHash) {
        self.intents_executed.push(AccountEvent::new(
            signer_id.to_owned(),
            IntentExecutedEvent { hash },
        ));
    }
}
