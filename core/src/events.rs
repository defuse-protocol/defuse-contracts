use std::borrow::Cow;

use defuse_serde_utils::base58::Base58;
use derive_more::derive::From;
use near_sdk::{near, serde::Deserialize, CryptoHash};
use serde_with::serde_as;

use crate::{
    accounts::{AccountEvent, PublicKeyEvent},
    fees::{FeeChangedEvent, FeeCollectorChangedEvent},
    intents::{token_diff::TokenDiff, tokens::Transfer, IntentExecutedEvent},
};

#[must_use = "make sure to `.emit()` this event"]
#[near(event_json(standard = "dip4"))]
#[derive(Debug, Clone, Deserialize, From)]
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
    Transfer(Cow<'a, [IntentEvent<AccountEvent<'a, Cow<'a, Transfer>>>]>),

    // TODO: add fee_collected event: both for distribution channel & protocol
    // TODO: add intent hash inside token_diff
    #[event_version("0.1.0")]
    TokenDiff(IntentEvent<AccountEvent<'a, Cow<'a, TokenDiff>>>),

    // TODO: just IntentEvent<AccountEvent<'a, ()>>?
    #[event_version("0.1.0")]
    IntentsExecuted(Cow<'a, [AccountEvent<'a, IntentExecutedEvent>]>),
}

#[must_use = "make sure to `.emit()` this event"]
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
pub struct IntentEvent<T> {
    #[serde_as(as = "Base58")]
    pub intent_hash: CryptoHash,
    #[serde(flatten)]
    pub event: T,
}

impl<T> IntentEvent<T> {
    #[inline]
    pub const fn new(event: T, intent_hash: CryptoHash) -> Self {
        Self { event, intent_hash }
    }
}

pub trait DefuseIntentEmit<'a>: Into<DefuseEvent<'a>> {
    #[inline]
    fn emit(self) {
        DefuseEvent::emit(&self.into())
    }
}
impl<'a, T> DefuseIntentEmit<'a> for T where T: Into<DefuseEvent<'a>> {}
