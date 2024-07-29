use near_sdk::near;

use super::{IntentId, LostAsset, SwapIntent};

#[must_use = "don't forget to `.emit()` this event"]
#[near(event_json(standard = "dip2"))]
pub enum Dip2Event<'a> {
    #[event_version("0.1.0")]
    Created {
        intent_id: &'a IntentId,
        #[serde(flatten)]
        intent: &'a SwapIntent,
    },
    #[event_version("0.1.0")]
    Executed(&'a IntentId),
    #[event_version("0.1.0")]
    RolledBack(&'a IntentId),
    #[event_version("0.1.0")]
    Lost {
        intent_id: &'a IntentId,
        #[serde(flatten)]
        asset: &'a LostAsset,
    },
    #[event_version("0.1.0")]
    Found(&'a IntentId),
}
