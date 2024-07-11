use defuse_contracts::{
    intents::swap::{
        events::Dep2Event, Asset, CreateSwapIntentAction, ExecuteSwapIntentAction, FtAmount,
        IntentId, LostAsset, SwapIntent, SwapIntentContract, SwapIntentError, SwapIntentStatus,
    },
    utils::{JsonLog, Mutex},
};

use near_sdk::{
    env, near,
    serde_json::{self, json},
    store::lookup_map::LookupMap,
    AccountId, BorshStorageKey, Gas, PanicOnDefault, Promise, PromiseError, PromiseOrValue,
};

mod ft;
mod lost_found;
mod native;
mod nft;
mod rollback;

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct SwapIntentContractImpl {
    intents: LookupMap<IntentId, Mutex<SwapIntentStatus>>,
}

#[derive(BorshStorageKey)]
#[near(serializers = [borsh])]
enum Prefix {
    Intents,
}

#[near]
impl SwapIntentContractImpl {
    #[must_use]
    #[init]
    #[allow(clippy::use_self)]
    pub fn new() -> Self {
        Self {
            intents: LookupMap::new(Prefix::Intents),
        }
    }
}

#[near]
impl SwapIntentContract for SwapIntentContractImpl {
    fn get_swap_intent(&self, id: &IntentId) -> Option<&Mutex<SwapIntentStatus>> {
        self.intents.get(id)
    }
}

impl SwapIntentContractImpl {
    fn create_intent(
        &mut self,
        sender: AccountId,
        asset_in: Asset,
        create: CreateSwapIntentAction,
    ) -> Result<(), SwapIntentError> {
        if create.expiration.has_expired() {
            return Err(SwapIntentError::Expired);
        }

        // prevent from swapping to zero amount
        if match create.asset_out {
            Asset::Native(amount) => amount.is_zero(),
            Asset::Ft(FtAmount { amount, .. }) => amount.0 == 0,
            Asset::Nft(_) => {
                // we can't know price of NFT
                false
            }
        } {
            return Err(SwapIntentError::ZeroAmount);
        }

        // TODO: storage management suggestions:
        //   * allow creating intents **starting** from only whitelisted assets
        //     which must have a non-zero price. This would prevent initiators
        //     from locking their funds in created intents by having an
        //     incentive to withdraw them back and free allocated storage
        //   * to create an intent from an asset that is not whitelisted,
        //     users must call storage_deposit according to
        //     Storage Management standard (NEP-145)

        let intent = SwapIntent {
            initiator: sender,
            asset_in,
            asset_out: create.asset_out,
            recipient: create.recipient,
            expiration: create.expiration,
        };

        Dep2Event::Created(&intent)
            .emit()
            .map_err(SwapIntentError::JSON)?;

        if self
            .intents
            .insert(create.id, SwapIntentStatus::Available(intent).into())
            .is_some()
        {
            return Err(SwapIntentError::AlreadyExists);
        }

        Ok(())
    }

    fn execute_intent(
        &mut self,
        sender: AccountId,
        received: Asset,
        execute: ExecuteSwapIntentAction,
    ) -> Result<Promise, SwapIntentError> {
        let intent = self
            .intents
            .get_mut(&execute.id)
            .ok_or(SwapIntentError::NotFound)?
            .lock()
            .and_then(|status| status.as_available())
            .ok_or(SwapIntentError::WrongStatus)?;

        if intent.has_expired() {
            // TODO: auto-rollback?
            return Err(SwapIntentError::Expired);
        }
        if received != intent.asset_out {
            return Err(SwapIntentError::WrongAssetOrAmount);
        }

        Ok(Self::transfer(
            &execute.id,
            intent.asset_in.clone(),
            execute.recipient.as_ref().unwrap_or(&sender).clone(),
        )
        .and(Self::transfer(
            &execute.id,
            // asset_out
            received,
            intent
                .recipient
                .as_ref()
                .unwrap_or(&intent.initiator)
                .clone(),
        ))
        .then(
            Self::ext(env::current_account_id())
                .with_static_gas(Self::GAR_FOR_RESOLVE_EXECUTE_TRANSFERS.saturating_add(
                    if intent.asset_out.is_native() {
                        // native asset should be refunded manualy
                        Asset::GAS_FOR_NATIVE_TRANSFER
                    } else {
                        // other assets have already reserved some gas
                        // for *_resolve_transfer() stage
                        Gas::from_gas(0)
                    },
                ))
                .resolve_execute_transfers(&execute.id, sender, execute.recipient),
        ))
    }

    /// Transfer asset to recipient (with static gas)
    #[inline]
    fn transfer(id: &IntentId, asset: Asset, recipient: AccountId) -> Promise {
        match asset {
            Asset::Native(amount) => Self::transfer_native(amount, recipient),
            Asset::Ft(ft) => Self::transfer_ft(ft, recipient, format!("Swap Intent '{id}'")),
            Asset::Nft(nft) => Self::transfer_nft(nft, recipient, format!("Swap Intent '{id}'")),
        }
    }
}

#[near]
impl SwapIntentContractImpl {
    // TODO: more accurate values
    const GAR_FOR_RESOLVE_EXECUTE_TRANSFERS: Gas = Gas::from_tgas(10);

    #[private]
    pub fn resolve_execute_transfers(
        &mut self,
        id: &IntentId,
        asset_out_sender: AccountId,
        asset_in_recipient: Option<AccountId>,
        #[callback_result] transfer_asset_in: &Result<(), PromiseError>,
        #[callback_result] transfer_asset_out: &Result<(), PromiseError>,
    ) -> PromiseOrValue<serde_json::Value> {
        self.internal_resolve_execute_transfers(
            id,
            asset_out_sender,
            asset_in_recipient,
            transfer_asset_in.is_ok(),
            transfer_asset_out.is_ok(),
        )
        .unwrap()
    }
}

impl SwapIntentContractImpl {
    fn internal_resolve_execute_transfers(
        &mut self,
        id: &IntentId,
        asset_out_sender: AccountId,
        asset_in_recipient: Option<AccountId>,
        transfer_asset_in_succeeded: bool,
        transfer_asset_out_succeeded: bool,
    ) -> Result<PromiseOrValue<serde_json::Value>, SwapIntentError> {
        let intent = self
            .intents
            .get_mut(id)
            .ok_or(SwapIntentError::NotFound)?
            .unlock()
            .ok_or(SwapIntentError::WrongStatus)?;

        let swap = intent.as_available().ok_or(SwapIntentError::WrongStatus)?;
        let asset_out = swap.asset_out.clone();

        if transfer_asset_in_succeeded && transfer_asset_out_succeeded {
            // both transfers succeeded
            self.intents.remove(id);
            Dep2Event::Executed(id)
                .emit()
                .map_err(SwapIntentError::JSON)?;
        } else if transfer_asset_in_succeeded ^ transfer_asset_out_succeeded {
            // exactly one of two transfers succeeded
            let lost = if transfer_asset_in_succeeded {
                // transfer_asset_out failed
                LostAsset {
                    asset: swap.asset_out.clone(),
                    recipient: swap.recipient.as_ref().unwrap_or(&swap.initiator).clone(),
                }
            } else {
                LostAsset {
                    asset: swap.asset_in.clone(),
                    recipient: asset_in_recipient.unwrap_or_else(|| asset_out_sender.clone()),
                }
            };
            Dep2Event::Lost {
                intent_id: id,
                asset: &lost,
            }
            .emit()
            .map_err(SwapIntentError::JSON)?;
            *intent = SwapIntentStatus::Lost(lost);
        }

        Ok(Self::maybe_refund(
            &asset_out,
            // refund only if both transfers failed
            !(transfer_asset_in_succeeded || transfer_asset_out_succeeded),
            asset_out_sender,
        ))
    }

    #[inline]
    fn maybe_refund(
        asset: &Asset,
        refund: bool,
        refund_to: AccountId,
    ) -> PromiseOrValue<serde_json::Value> {
        match asset {
            // native_action()
            Asset::Native(amount) => {
                if refund {
                    Promise::new(refund_to).transfer(*amount).into()
                } else {
                    PromiseOrValue::Value(json!(true))
                }
            }
            // ft_on_transfer()
            Asset::Ft(FtAmount { amount, .. }) => {
                PromiseOrValue::Value(json!(if refund { *amount } else { 0.into() }))
            }
            // nft_on_transfer()
            Asset::Nft(_) => PromiseOrValue::Value(json!(refund)),
        }
    }
}
