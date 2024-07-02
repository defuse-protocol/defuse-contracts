use defuse_contracts::{
    intents::swap::{events::Dep2Event, IntentId, LostAsset, LostFound, SwapIntentError},
    utils::JsonLog,
};
use near_sdk::{env, near, Gas, NearToken, Promise, PromiseError};

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
    fn internal_lost_found(&mut self, id: &IntentId) -> Result<Promise, SwapIntentError> {
        let LostAsset { asset, recipient } = self
            .intents
            .get_mut(id)
            .ok_or(SwapIntentError::NotFound)?
            .lock()
            .and_then(|status| status.as_lost())
            .ok_or(SwapIntentError::WrongStatus)?;

        Ok(Self::transfer(id, asset.clone(), recipient.clone()).then(
            Self::ext(env::current_account_id())
                .with_static_gas(Self::GAS_FOR_RESOLVE_LOST_FOUND)
                .resolve_lost_found(id),
        ))
    }
}

#[near]
impl SwapIntentContractImpl {
    const GAS_FOR_RESOLVE_LOST_FOUND: Gas = Gas::from_tgas(1);

    #[private]
    pub fn resolve_lost_found(
        &mut self,
        id: &IntentId,
        #[callback_result] transfer: &Result<(), PromiseError>,
    ) -> bool {
        self.internal_resolve_lost_found(id, transfer.is_ok())
            .unwrap()
    }
}

impl SwapIntentContractImpl {
    fn internal_resolve_lost_found(
        &mut self,
        id: &IntentId,
        transfer_succeeded: bool,
    ) -> Result<bool, SwapIntentError> {
        self.intents
            .get_mut(id)
            .ok_or(SwapIntentError::NotFound)?
            .unlock()
            .and_then(|status| status.as_lost())
            .ok_or(SwapIntentError::WrongStatus)?;

        if transfer_succeeded {
            self.intents.remove(id);
            Dep2Event::Found(id).emit().map_err(SwapIntentError::JSON)?;
        }

        Ok(transfer_succeeded)
    }
}
