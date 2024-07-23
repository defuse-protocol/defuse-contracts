use near_sdk::serde::Serialize;

use crate::utils::Nep297Event;

use super::{IntentId, LostAsset, SwapIntent};

#[derive(Debug, Serialize)]
#[serde(crate = "near_sdk::serde")]
#[must_use = "don't forget to `.emit()` this event"]
#[serde(tag = "event", content = "data", rename_all = "snake_case")]
pub enum Dip2Event<'a> {
    Created(&'a SwapIntent),
    Executed(&'a IntentId),
    RolledBack(&'a IntentId),
    Lost {
        intent_id: &'a IntentId,
        #[serde(flatten)]
        asset: &'a LostAsset,
    },
    Found(&'a IntentId),
}

impl<'a> From<Dip2Event<'a>> for Nep297Event<Dip2Event<'a>> {
    fn from(data: Dip2Event<'a>) -> Self {
        Self {
            standard: "dip2",
            version: "0.1.0",
            event: data,
        }
    }
}
