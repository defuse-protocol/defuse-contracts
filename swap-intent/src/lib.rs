use defuse_contracts::{
    intents::swap::{
        events::Dep2Event, Asset, CreateSwapIntentAction, ExecuteSwapIntentAction, FtAmount,
        IntentId, LostAsset, SwapIntent, SwapIntentContract, SwapIntentError, SwapIntentStatus,
    },
    utils::{JsonLog, Mutex},
};

use near_sdk::{
    env,
    json_types::U128,
    near,
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
        if create.deadline.has_expired() {
            return Err(SwapIntentError::Expired);
        }

        // prevent from swapping to zero amount
        if match create.asset_out {
            Asset::Native(amount) => amount.is_zero(),
            Asset::Ft(FtAmount { amount, .. }) => amount == 0,
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
            deadline: create.deadline,
        };

        Dep2Event::Created(&intent)
            .log_json()
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
            return Err(SwapIntentError::WrongAsset);
        }

        Ok(Self::transfer(
            &execute.id,
            // asset_out
            received,
            intent
                .recipient
                .as_ref()
                .unwrap_or(&intent.initiator)
                .clone(),
        )
        .then(
            Self::ext(env::current_account_id())
                .with_static_gas(
                    Self::GAS_FOR_RESOLVE_TRANSFER_ASSET_OUT
                        .saturating_add(intent.asset_in.gas_for_transfer())
                        .saturating_add(Self::GAS_FOR_RESOLVE_TRANSFER_ASSET_IN),
                )
                .resolve_transfer_asset_out(&execute.id, sender, execute.recipient),
        ))
    }

    /// Transfer asset to recipient (with static gas)
    #[inline]
    pub(crate) fn transfer(id: &IntentId, asset: Asset, recipient: AccountId) -> Promise {
        match asset {
            Asset::Native(amount) => Self::transfer_native(amount, recipient),
            Asset::Ft(ft) => Self::transfer_ft(ft, recipient, format!("Swap Intent '{id}'")),
            Asset::Nft(nft) => Self::transfer_nft(nft, recipient, format!("Swap Intent '{id}'")),
        }
    }
}

#[near]
impl SwapIntentContractImpl {
    // TODO: more accurate value
    const GAS_FOR_RESOLVE_TRANSFER_ASSET_OUT: Gas = Gas::from_tgas(5);

    #[private]
    pub fn resolve_transfer_asset_out(
        &mut self,
        id: &IntentId,
        asset_out_sender: AccountId,
        #[callback_result] transfer_asset_out: &Result<(), PromiseError>,
        asset_in_recipient: Option<AccountId>,
    ) -> PromiseOrValue<serde_json::Value> {
        self.internal_resolve_transfer_asset_out(
            id,
            asset_out_sender,
            transfer_asset_out.is_ok(),
            asset_in_recipient,
        )
        .unwrap()
    }
}

impl SwapIntentContractImpl {
    fn internal_resolve_transfer_asset_out(
        &mut self,
        id: &IntentId,
        asset_out_sender: AccountId,
        transfer_asset_out_succeeded: bool,
        asset_in_recipient: Option<AccountId>,
    ) -> Result<PromiseOrValue<serde_json::Value>, SwapIntentError> {
        let intent = self.intents.get_mut(id).ok_or(SwapIntentError::NotFound)?;

        if !transfer_asset_out_succeeded {
            let intent = intent
                .unlock()
                .and_then(|status| status.as_available())
                .ok_or(SwapIntentError::WrongStatus)?;
            return Ok(match intent.asset_out {
                Asset::Native(_) => {
                    // Native transfer can fail only if we don't have enough NEAR.
                    // Since we create native intents with the exact amount
                    // of attached NEAR, this situation can only happen if
                    // we run out of NEAR due to incorrect storage management
                    // while creating new intents.
                    unreachable!()
                }
                Asset::Ft(FtAmount { amount, .. }) => {
                    // return back to sender: ft_on_transfer(asset_out)
                    PromiseOrValue::Value(json!(U128(amount)))
                }
                Asset::Nft(_) => {
                    // return back to sender: nft_on_transfer(asset_out)
                    PromiseOrValue::Value(json!(true))
                }
            });
        }

        let intent = intent
            .as_locked()
            .and_then(|status| status.as_available())
            .ok_or(SwapIntentError::WrongStatus)?;
        let asset_in_recipient = asset_in_recipient.unwrap_or(asset_out_sender);

        Ok(
            Self::transfer(id, intent.asset_in.clone(), asset_in_recipient.clone())
                .then(
                    Self::ext(env::current_account_id())
                        .resolve_transfer_asset_in(id, asset_in_recipient),
                )
                .into(),
        )
    }
}

#[near]
impl SwapIntentContractImpl {
    // TODO: more accurate value
    const GAS_FOR_RESOLVE_TRANSFER_ASSET_IN: Gas = Gas::from_tgas(5);

    #[private]
    pub fn resolve_transfer_asset_in(
        &mut self,
        id: &IntentId,
        #[callback_result] transfer_asset_in: &Result<(), PromiseError>,
        asset_in_recipient: AccountId,
    ) -> serde_json::Value {
        self.internal_resolve_transfer_asset_in(id, transfer_asset_in.is_ok(), asset_in_recipient)
            .unwrap()
    }
}

impl SwapIntentContractImpl {
    fn internal_resolve_transfer_asset_in(
        &mut self,
        id: &IntentId,
        transfer_asset_in_succeeded: bool,
        asset_in_recipient: AccountId,
    ) -> Result<serde_json::Value, SwapIntentError> {
        let intent = self
            .intents
            .get_mut(id)
            .ok_or(SwapIntentError::NotFound)?
            .unlock()
            .ok_or(SwapIntentError::WrongStatus)?;

        let swap = intent.as_available().ok_or(SwapIntentError::WrongStatus)?;
        let asset_out = swap.asset_out.clone();

        if transfer_asset_in_succeeded {
            self.intents.remove(id);
        } else {
            let lost = LostAsset {
                asset: swap.asset_in.clone(),
                recipient: asset_in_recipient,
            };
            Dep2Event::Lost {
                intent_id: id,
                asset: &lost,
            }
            .log_json()
            .map_err(SwapIntentError::JSON)?;
            *intent = SwapIntentStatus::Lost(lost);
        }

        Dep2Event::Executed(id)
            .log_json()
            .map_err(SwapIntentError::JSON)?;

        Ok(match asset_out {
            // native_action(asset_out)
            Asset::Native(_) => json!(true),
            // ft_on_transfer(asset_out)
            Asset::Ft(_) => json!(U128(0)),
            // nft_on_transfer(asset_out)
            Asset::Nft(_) => json!(false),
        })
    }
}
