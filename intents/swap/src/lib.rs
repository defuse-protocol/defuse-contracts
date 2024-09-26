use defuse_contracts::{
    intents::swap::{
        events::Dip2Event, AssetWithAccount, CreateSwapIntentAction, ExecuteSwapIntentAction,
        FtAmount, GenericAccount, IntentId, LostAsset, NearAsset, SwapIntent, SwapIntentContract,
        SwapIntentError, SwapIntentStatus,
    },
    utils::{Lock, UnwrapOrPanic},
};

use near_sdk::{
    env, near,
    serde_json::{self, json},
    store::lookup_map::LookupMap,
    BorshStorageKey, Gas, PanicOnDefault, Promise, PromiseError, PromiseOrValue,
};

mod cross_chain;
mod ft;
mod lost_found;
mod native;
mod nft;
mod rollback;

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct SwapIntentContractImpl {
    intents: LookupMap<IntentId, Lock<SwapIntent>>,
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
    fn get_intent(&self, id: &IntentId) -> Option<&Lock<SwapIntent>> {
        self.intents.get(id)
    }
}

impl SwapIntentContractImpl {
    const INTENT_ID_MAX_LENGTH: usize = 128;

    fn create_intent(
        &mut self,
        asset_in: AssetWithAccount,
        create: CreateSwapIntentAction,
    ) -> Result<(), SwapIntentError> {
        // TODO: storage management suggestions:
        //   * allow creating intents **starting** from only whitelisted assets
        //     which must have a non-zero price. This would prevent initiators
        //     from locking their funds in created intents by having an
        //     incentive to withdraw them back and free allocated storage
        //   * to create an intent from an asset that is not whitelisted,
        //     users must call storage_deposit according to
        //     Storage Management standard (NEP-145)

        if create.id.len() > Self::INTENT_ID_MAX_LENGTH {
            return Err(SwapIntentError::IntentIdTooLong);
        }

        let intent = SwapIntent {
            asset_in,
            asset_out: create.asset_out,
            lockup_until: create.lockup_until,
            expiration: create.expiration,
            status: SwapIntentStatus::Available,
            lost: None,
            referral: create.referral,
        };
        intent.validate()?;

        Dip2Event::Created {
            intent_id: &create.id,
            intent: &intent,
        }
        .emit();

        if self.intents.insert(create.id, intent.into()).is_some() {
            return Err(SwapIntentError::AlreadyExists);
        }

        Ok(())
    }

    fn execute_intent(
        &mut self,
        received: &AssetWithAccount,
        execute: ExecuteSwapIntentAction,
    ) -> Result<Promise, SwapIntentError> {
        let intent = self
            .intents
            .get_mut(&execute.id)
            .ok_or(SwapIntentError::NotFound)?
            .lock()
            .filter(|intent| intent.is_available())
            .ok_or(SwapIntentError::WrongStatus)?;

        if intent.has_expired() {
            // TODO: auto-rollback?
            return Err(SwapIntentError::Expired);
        }
        if received.asset() != intent.asset_out.asset() {
            return Err(SwapIntentError::WrongAssetOut);
        }

        // TODO: for trustless cross-chain swaps, we need first
        // to lock funds on all chains and only after they have
        // been locked, initiate cross-chain swap. This is needed
        // to keep chains synced and avoid intent rollback on foreign
        // chain while being swapped.

        // TODO: or step-by-step transfers to ensure optional storage deposit by solver?
        Ok(Self::transfer(
            &execute.id,
            intent
                .asset_in
                .with_account(execute.recipient.clone())
                .ok_or(SwapIntentError::InvalidRecipient)?,
        )
        .and(Self::transfer(&execute.id, intent.asset_out.clone()))
        .then(
            Self::ext(env::current_account_id())
                .with_static_gas(
                    Self::GAR_FOR_RESOLVE_EXECUTE_TRANSFERS
                        // reserve gas to refund asset_out in case both
                        // transfers failed
                        .saturating_add(intent.asset_out.gas_for_refund()),
                )
                .resolve_execute_transfers(&execute.id, execute.recipient, received, execute.proof),
        ))
    }

    /// Transfer asset to recipient (with static gas)
    #[inline]
    fn transfer(id: &IntentId, asset: AssetWithAccount) -> Promise {
        match asset {
            AssetWithAccount::Near {
                account: recipient,
                asset,
            } => match asset {
                NearAsset::Native { amount } => Self::transfer_native(amount, recipient),
                NearAsset::Nep141(ft) => {
                    Self::transfer_ft(ft, recipient, format!("Swap Intent '{id}'"))
                }
                NearAsset::Nep171(nft) => {
                    Self::transfer_nft(nft, recipient, format!("Swap Intent '{id}'"))
                }
            },
            AssetWithAccount::CrossChain {
                account: recipient,
                asset,
            } => Self::transfer_cross_chain_asset(asset, recipient),
        }
    }
}

#[near]
impl SwapIntentContractImpl {
    // TODO: more accurate values
    const GAR_FOR_RESOLVE_EXECUTE_TRANSFERS: Gas = Gas::from_tgas(50);

    #[private]
    pub fn resolve_execute_transfers(
        &mut self,
        id: &IntentId,
        asset_in_recipient: GenericAccount,
        asset_out_received: &AssetWithAccount,
        asset_out_proof: Option<String>,
        #[callback_result] transfer_asset_in: &Result<(), PromiseError>,
        #[callback_result] transfer_asset_out: &Result<(), PromiseError>,
    ) -> PromiseOrValue<serde_json::Value> {
        self.internal_resolve_execute_transfers(
            id,
            asset_in_recipient,
            asset_out_received,
            asset_out_proof,
            transfer_asset_in.is_ok(),
            transfer_asset_out.is_ok(),
        )
        .unwrap_or_panic_display()
    }
}

impl SwapIntentContractImpl {
    fn internal_resolve_execute_transfers(
        &mut self,
        id: &IntentId,
        asset_in_recipient: GenericAccount,
        asset_out_received: &AssetWithAccount,
        asset_out_proof: Option<String>,
        transfer_asset_in_succeeded: bool,
        transfer_asset_out_succeeded: bool,
    ) -> Result<PromiseOrValue<serde_json::Value>, SwapIntentError> {
        let intent = self
            .intents
            .get_mut(id)
            .ok_or(SwapIntentError::NotFound)?
            .unlock()
            .ok_or(SwapIntentError::WrongStatus)?;

        if transfer_asset_in_succeeded || transfer_asset_out_succeeded {
            let maybe_lost = if transfer_asset_in_succeeded && transfer_asset_out_succeeded {
                None
            } else if !transfer_asset_in_succeeded {
                Some(LostAsset::AssetIn {
                    recipient: asset_in_recipient,
                })
            } else {
                Some(LostAsset::AssetOut)
            };
            // TODO: remove the intent to free the storage
            intent.set_executed(id, asset_out_proof, maybe_lost);
        }

        Ok(Self::maybe_refund(
            asset_out_received,
            // refund only if both transfers failed
            !(transfer_asset_in_succeeded || transfer_asset_out_succeeded),
        ))
    }

    #[inline]
    fn maybe_refund(asset: &AssetWithAccount, refund: bool) -> PromiseOrValue<serde_json::Value> {
        match asset {
            AssetWithAccount::Near {
                account: refund_to,
                asset,
            } => match asset {
                // native_action()
                NearAsset::Native { amount } => {
                    if refund {
                        Promise::new(refund_to.clone()).transfer(*amount).into()
                        // TODO: return false?: env::value_return(value)
                    } else {
                        PromiseOrValue::Value(json!(true))
                    }
                }
                // ft_on_transfer()
                NearAsset::Nep141(FtAmount { amount, .. }) => {
                    PromiseOrValue::Value(json!(if refund { *amount } else { 0.into() }))
                }
                // nft_on_transfer()
                NearAsset::Nep171(_) => PromiseOrValue::Value(json!(refund)),
            },
            // cross_chain_on_transfer()
            AssetWithAccount::CrossChain { .. } => PromiseOrValue::Value(json!(
                // TODO: fix to `refund`, so it would return if the asset
                // should be refunded
                !refund
            )),
        }
    }
}
