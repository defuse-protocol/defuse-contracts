use defuse_contracts::{
    intents::swap::{Asset, IntentId, SwapError},
    utils::Mutex,
};
use near_sdk::{near, store::LookupMap, AccountId, IntoStorageKey};

#[derive(Debug)]
#[near(serializers = [borsh])]
pub struct LostFound(LookupMap<IntentId, (Asset, AccountId)>);

impl LostFound {
    pub fn new<K>(key_prefix: K) -> Self
    where
        K: IntoStorageKey,
    {
        Self(LookupMap::new(key_prefix))
    }

    pub fn lost(&mut self, id: IntentId, asset: Asset, recipient: AccountId) {
        // TODO: what if already exists
        // TODO: emit log
        self.0.insert(id, (asset, recipient));
    }

    // pub fn lock(&mut self, id: &IntentID) -> Result<&(Asset, AccountId), SwapError> {
    //     self.0
    //         .get_mut(id)
    //         .ok_or_else(|| SwapError::NotFound(id.clone()))?
    //         .lock()
    //         .ok_or(SwapError::Locked)
    // }

    pub fn found(&mut self, id: &IntentId) -> Option<(Asset, AccountId)> {
        self.0.remove(id)
    }
}

// pub struct Lost {
//     pub intent_id: IntentID,

// }
