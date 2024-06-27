use defuse_contracts::{
    intents::swap::{events::Dep2Event, IntentId, LostAsset, LostFound, SwapError},
    utils::JsonLog,
};
use near_sdk::{env, near, NearToken, Promise, PromiseError};

use crate::{SwapIntentContractImpl, SwapIntentContractImplExt};

#[near]
impl LostFound for SwapIntentContractImpl {
    #[payable]
    fn lost_found(&mut self, id: &IntentId) -> Promise {
        assert_eq!(env::attached_deposit(), NearToken::from_yoctonear(1));
        self.internal_lost_found(id).unwrap()
    }
}

impl SwapIntentContractImpl {
    fn internal_lost_found(&mut self, id: &IntentId) -> Result<Promise, SwapError> {
        let LostAsset { asset, recipient } = self
            .intents
            .get_mut(id)
            .ok_or_else(|| SwapError::NotFound(id.clone()))?
            .lock()
            .ok_or(SwapError::Locked)?
            .as_lost()
            .ok_or(SwapError::WrongStatus)?;

        Ok(Self::transfer(id, asset.clone(), recipient.clone()).then(
            Self::ext(env::current_account_id())
                // TODO: check if enough gas
                .resolve_lost_found(id),
        ))
    }
}

#[near]
impl SwapIntentContractImpl {
    #[private]
    pub fn resolve_lost_found(
        &mut self,
        id: &IntentId,
        #[callback_result] transfer: &Result<(), PromiseError>,
    ) -> bool {
        self.internal_resolve_lost_found(id, transfer).unwrap()
    }
}

impl SwapIntentContractImpl {
    fn internal_resolve_lost_found(
        &mut self,
        id: &IntentId,
        transfer: &Result<(), PromiseError>,
    ) -> Result<bool, SwapError> {
        self.intents
            .get_mut(id)
            .ok_or_else(|| SwapError::NotFound(id.clone()))?
            .unlock()
            .ok_or(SwapError::Unlocked)?
            .as_lost()
            .ok_or(SwapError::WrongStatus)?;

        if transfer.is_ok() {
            self.intents.remove(id);
            Dep2Event::Found(id).log_json().map_err(SwapError::JSON)?;
        }
        Ok(transfer.is_ok())
    }
}
