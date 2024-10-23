use defuse_contracts::{
    intents::swap::{IntentId, Rollback, SwapIntentError},
    utils::UnwrapOrPanicError,
};
use near_sdk::{env, near, AccountId, Gas, PromiseError, PromiseOrValue};

use crate::{SwapIntentContractImpl, SwapIntentContractImplExt};

#[near]
impl Rollback for SwapIntentContractImpl {
    #[payable]
    fn rollback_intent(&mut self, id: &IntentId) -> PromiseOrValue<bool> {
        near_sdk::assert_one_yocto();
        self.internal_rollback_intent(id, &env::predecessor_account_id())
            .unwrap_or_panic_display()
    }
}

impl SwapIntentContractImpl {
    fn internal_rollback_intent(
        &mut self,
        id: &IntentId,
        initiator: &AccountId,
    ) -> Result<PromiseOrValue<bool>, SwapIntentError> {
        let intent = self
            .intents
            .get_mut(id)
            .ok_or(SwapIntentError::NotFound)?
            .lock()
            .filter(|intent| intent.is_available())
            .ok_or(SwapIntentError::WrongStatus)?;

        if initiator != intent.initiator() {
            return Err(SwapIntentError::Unauthorized);
        }
        if intent.is_locked_up() {
            return Err(SwapIntentError::LockedUp);
        }

        Ok(Self::transfer(id, intent.asset_in.clone())
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Self::GAS_FOR_RESOLVE_ROLLBACK_INTENT)
                    .resolve_rollback_intent(id),
            )
            .into())
    }
}

#[near]
impl SwapIntentContractImpl {
    const GAS_FOR_RESOLVE_ROLLBACK_INTENT: Gas = Gas::from_tgas(1);

    #[private]
    pub fn resolve_rollback_intent(
        &mut self,
        id: &IntentId,
        #[callback_result] transfer_asset_in: &Result<(), PromiseError>,
    ) -> bool {
        self.internal_resolve_rollback_intent(id, transfer_asset_in.is_ok())
            .unwrap_or_panic_display()
    }
}

impl SwapIntentContractImpl {
    fn internal_resolve_rollback_intent(
        &mut self,
        id: &IntentId,
        transfer_asset_in_succeeded: bool,
    ) -> Result<bool, SwapIntentError> {
        let intent = self
            .intents
            .get_mut(id)
            .ok_or(SwapIntentError::NotFound)?
            .unlock()
            .ok_or(SwapIntentError::WrongStatus)?;

        intent.set_rolled_back(id, !transfer_asset_in_succeeded);

        Ok(transfer_asset_in_succeeded)
    }
}
