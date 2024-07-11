use defuse_contracts::{
    intents::swap::{
        events::Dep2Event, IntentId, LostAsset, Rollback, SwapIntentError, SwapIntentStatus,
    },
    utils::JsonLog,
};
use near_sdk::{env, near, AccountId, Gas, NearToken, PromiseError, PromiseOrValue};

use crate::{SwapIntentContractImpl, SwapIntentContractImplExt};

#[near]
impl Rollback for SwapIntentContractImpl {
    #[payable]
    fn rollback_intent(&mut self, id: &IntentId) -> PromiseOrValue<bool> {
        assert_eq!(env::attached_deposit(), NearToken::from_yoctonear(1));
        self.internal_rollback_intent(id, &env::predecessor_account_id())
            .unwrap()
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
            .and_then(|status| status.as_available())
            .ok_or(SwapIntentError::WrongStatus)?;

        if initiator != &intent.initiator {
            return Err(SwapIntentError::Unauthorized);
        }

        Ok(
            Self::transfer(id, intent.asset_in.clone(), intent.initiator.clone())
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(Self::GAS_FOR_RESOLVE_ROLLBACK_INTENT)
                        .resolve_rollback_intent(id),
                )
                .into(),
        )
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
            .unwrap()
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

        let swap = intent.as_available().ok_or(SwapIntentError::WrongStatus)?;

        if transfer_asset_in_succeeded {
            self.intents.remove(id);
            Dep2Event::Rollbacked(id)
                .emit()
                .map_err(SwapIntentError::JSON)?;
        } else {
            let lost = LostAsset {
                asset: swap.asset_in.clone(),
                recipient: swap.initiator.clone(),
            };
            Dep2Event::Lost {
                intent_id: id,
                asset: &lost,
            }
            .emit()
            .map_err(SwapIntentError::JSON)?;
            *intent = SwapIntentStatus::Lost(lost);
        }
        Ok(transfer_asset_in_succeeded)
    }
}
