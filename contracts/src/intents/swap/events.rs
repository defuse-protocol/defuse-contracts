use near_sdk::serde::Serialize;

use super::{IntentId, LostAsset, SwapIntent};

#[derive(Debug, Serialize)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "event", content = "data", rename_all = "snake_case")]
pub enum Dep2Event<'a> {
    Created(&'a SwapIntent),
    Executed(&'a IntentId),
    Rollbacked(&'a IntentId),
    Lost {
        intent_id: &'a IntentId,
        #[serde(flatten)]
        asset: &'a LostAsset,
    },
    Found(&'a IntentId),
}
