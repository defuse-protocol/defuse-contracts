use defuse_contracts::intents::swap::{IntentId, Lost, LostFound, SwapError};
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
        let Lost { asset, recipient } = self
            .intents
            .get_mut(id)
            .ok_or_else(|| SwapError::NotFound(id.clone()))?
            .lock_mut()
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
        #[callback_result] transfer: Result<(), PromiseError>,
    ) -> bool {
        let intent = self
            .intents
            .get_mut(id)
            .ok_or_else(|| SwapError::NotFound(id.clone()))
            .unwrap();
        intent
            .unlock_mut()
            .ok_or(SwapError::Unlocked)
            .unwrap()
            .as_lost()
            .ok_or(SwapError::WrongStatus)
            .unwrap();

        if transfer.is_ok() {
            self.intents.remove(id);
            return true;
        }

        false
    }
}
