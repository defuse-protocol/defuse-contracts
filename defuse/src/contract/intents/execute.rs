use std::borrow::Cow;

use defuse_core::{
    accounts::AccountEvent,
    engine::Inspector,
    events::DefuseEvent,
    intents::{token_diff::TokenDiff, tokens::Transfer, IntentExecutedEvent},
    Deadline,
};
use near_sdk::{AccountIdRef, CryptoHash};

// TODO: rename?
#[derive(Debug, Default)]
pub struct ExecuteInspector {
    pub intents_executed: Vec<AccountEvent<'static, IntentExecutedEvent>>,
}

impl Inspector for ExecuteInspector {
    #[inline]
    fn on_deadline(&mut self, _deadline: Deadline) {}

    #[inline]
    fn on_transfer(&mut self, sender_id: &AccountIdRef, transfer: &Transfer) {
        DefuseEvent::Transfer(AccountEvent::new(sender_id, Cow::Borrowed(transfer))).emit();
    }

    #[inline]
    fn on_token_diff(&mut self, owner_id: &AccountIdRef, token_diff: &TokenDiff) {
        DefuseEvent::TokenDiff(AccountEvent::new(owner_id, Cow::Borrowed(token_diff))).emit();
    }

    #[inline]
    fn on_intent_executed(&mut self, signer_id: &AccountIdRef, hash: CryptoHash) {
        self.intents_executed.push(AccountEvent::new(
            Cow::Owned(signer_id.to_owned()),
            IntentExecutedEvent { hash },
        ));
    }
}

// TODO: or .emit() method?
impl Drop for ExecuteInspector {
    fn drop(&mut self) {
        if !self.intents_executed.is_empty() {
            DefuseEvent::IntentsExecuted(self.intents_executed.as_slice().into()).emit();
        }
    }
}
