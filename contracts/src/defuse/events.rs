use std::borrow::Cow;

use derive_more::derive::From;
use near_sdk::near;

use super::{
    accounts::PublicKeyEvent,
    fees::{FeeChangedEvent, FeeCollectorChangedEvent},
    intents::{token_diff::TokenDiff, AccountEvent, IntentExecutedEvent},
};

#[must_use = "make sure to `.emit()` this event"]
#[near(event_json(standard = "dip4"))]
#[derive(Debug, From)]
pub enum DefuseEvent<'a> {
    #[event_version("0.1.0")]
    #[from(skip)]
    PublicKeyAdded(AccountEvent<'a, PublicKeyEvent<'a>>),
    #[event_version("0.1.0")]
    #[from(skip)]
    PublicKeyRemoved(AccountEvent<'a, PublicKeyEvent<'a>>),

    #[event_version("0.1.0")]
    FeeChanged(FeeChangedEvent),
    #[event_version("0.1.0")]
    FeeCollectorChanged(FeeCollectorChangedEvent<'a>),

    #[event_version("0.1.0")]
    IntentsExecuted(&'a [AccountEvent<'a, IntentExecutedEvent>]),

    #[event_version("0.1.0")]
    TokenDiff(AccountEvent<'a, Cow<'a, TokenDiff>>),
}

pub trait DefuseIntentEmit<'a>: Into<DefuseEvent<'a>> {
    #[inline]
    fn emit(self) {
        DefuseEvent::emit(&self.into())
    }
}
impl<'a, T> DefuseIntentEmit<'a> for T where T: Into<DefuseEvent<'a>> {}
