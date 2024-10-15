use derive_more::derive::From;
use near_sdk::near;

use super::{
    accounts::{PublicKeyAddedEvent, PublicKeyRemovedEvent},
    fees::{FeeChangedEvent, FeeCollectorChangedEvent},
    intents::{token_diff::TokenDiffEvent, IntentExecutedEvent},
};

#[must_use = "make sure to `.emit()` this event"]
#[near(event_json(standard = "dip4"))]
#[derive(Debug, From)]
pub enum DefuseEvent<'a> {
    #[event_version("0.1.0")]
    PublicKeyAdded(PublicKeyAddedEvent<'a>),
    #[event_version("0.1.0")]
    PublicKeyRemoved(PublicKeyRemovedEvent<'a>),

    #[event_version("0.1.0")]
    FeeChanged(FeeChangedEvent<'a>),
    #[event_version("0.1.0")]
    FeeCollectorChanged(FeeCollectorChangedEvent<'a>),

    #[event_version("0.1.0")]
    IntentExecuted(IntentExecutedEvent<'a>),

    #[event_version("0.1.0")]
    TokenDiff(TokenDiffEvent<'a>),
}

pub trait DefuseIntentEmit<'a>: Into<DefuseEvent<'a>> {
    #[inline]
    fn emit(self) {
        DefuseEvent::emit(&self.into())
    }
}
impl<'a, T> DefuseIntentEmit<'a> for T where T: Into<DefuseEvent<'a>> {}
