use defuse_contracts::{
    intents::swap::{IntentId, LostFound, SwapIntentError},
    utils::UnwrapOrPanic,
};
use near_sdk::{env, near, Gas, Promise, PromiseError};

use crate::{SwapIntentContractImpl, SwapIntentContractImplExt};

#[near]
impl LostFound for SwapIntentContractImpl {
    #[payable]
    fn lost_found(&mut self, id: &IntentId) -> Promise {
        near_sdk::assert_one_yocto();
        self.internal_lost_found(id).unwrap_or_panic_display()
    }
}

impl SwapIntentContractImpl {
    fn internal_lost_found(&mut self, id: &IntentId) -> Result<Promise, SwapIntentError> {
        let intent = self
            .intents
            .get_mut(id)
            .ok_or(SwapIntentError::NotFound)?
            .lock()
            .ok_or(SwapIntentError::WrongStatus)?;

        let Some(asset) = intent.lost_asset() else {
            return Err(SwapIntentError::WrongStatus);
        };

        Ok(Self::transfer(id, asset).then(
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
            .unwrap_or_panic_display()
    }
}

impl SwapIntentContractImpl {
    fn internal_resolve_lost_found(
        &mut self,
        id: &IntentId,
        transfer_succeeded: bool,
    ) -> Result<bool, SwapIntentError> {
        let intent = self
            .intents
            .get_mut(id)
            .ok_or(SwapIntentError::NotFound)?
            .unlock()
            .ok_or(SwapIntentError::WrongStatus)?;

        if transfer_succeeded {
            intent.lost_found(id);
        }

        Ok(transfer_succeeded)
    }
}
