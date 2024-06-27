use defuse_contracts::{
    intents::swap::{
        events::Dep2Event, IntentId, LostAsset, Rollback, SwapIntentError, SwapIntentStatus,
    },
    utils::JsonLog,
};
use near_sdk::{env, near, AccountId, NearToken, PromiseError, PromiseOrValue};

use crate::{SwapIntentContractImpl, SwapIntentContractImplExt};

#[near]
impl Rollback for SwapIntentContractImpl {
    #[payable]
    fn rollback_intent(&mut self, id: &IntentId) -> PromiseOrValue<bool> {
        assert_eq!(env::attached_deposit(), NearToken::from_yoctonear(1));
        self.internal_rollback_intent(id, env::predecessor_account_id())
            .unwrap()
    }
}

impl SwapIntentContractImpl {
    fn internal_rollback_intent(
        &mut self,
        id: &IntentId,
        initiator: AccountId,
    ) -> Result<PromiseOrValue<bool>, SwapIntentError> {
        let intent = self
            .intents
            .get_mut(id)
            .ok_or(SwapIntentError::NotFound)?
            .lock()
            .and_then(|status| status.as_available())
            .ok_or(SwapIntentError::WrongStatus)?;

        if initiator != intent.initiator {
            return Err(SwapIntentError::Unauthorized);
        }

        assert!(
            env::prepaid_gas().saturating_sub(env::used_gas())
                >= intent.asset_in.gas_for_transfer()
        );

        Ok(
            Self::transfer(id, intent.asset_in.clone(), intent.initiator.clone())
                .then(Self::ext(env::current_account_id()).resolve_rollback_intent(id))
                .into(),
        )
    }
}

#[near]
impl SwapIntentContractImpl {
    #[private]
    pub fn resolve_rollback_intent(
        &mut self,
        id: &IntentId,
        #[callback_result] transfer_asset_in: Result<(), PromiseError>,
    ) -> bool {
        self.internal_resolve_rollback_intent(id, transfer_asset_in)
            .unwrap()
    }
}

impl SwapIntentContractImpl {
    fn internal_resolve_rollback_intent(
        &mut self,
        id: &IntentId,
        transfer_asset_in: Result<(), PromiseError>,
    ) -> Result<bool, SwapIntentError> {
        let intent = self
            .intents
            .get_mut(id)
            .ok_or(SwapIntentError::NotFound)?
            .unlock()
            .ok_or(SwapIntentError::WrongStatus)?;

        let swap = intent
            .as_available()
            .ok_or(SwapIntentError::WrongStatus)?
            .clone();

        if transfer_asset_in.is_ok() {
            Dep2Event::Rollbacked(id)
                .log_json()
                .map_err(SwapIntentError::JSON)?;
            self.intents.remove(id);
        } else {
            let lost = LostAsset {
                asset: swap.asset_in,
                recipient: swap.initiator,
            };
            Dep2Event::Lost {
                intent_id: id,
                asset: &lost,
            }
            .log_json()
            .map_err(SwapIntentError::JSON)?;
            *intent = SwapIntentStatus::Lost(lost);
        }
        Ok(transfer_asset_in.is_ok())
    }
}
