use defuse_contracts::{
    intents::swap::{
        Asset, CreateSwapIntentAction, ExecuteSwapIntentAction, FtAmount, IntentId, LostAsset,
        SwapError, SwapIntent, SwapIntentContract, SwapIntentStatus,
    },
    utils::Mutex,
};

use near_sdk::{
    env,
    json_types::U128,
    log, near,
    serde::Serialize,
    serde_json::{self, json},
    store::lookup_map::{Entry, LookupMap},
    AccountId, BorshStorageKey, Gas, PanicOnDefault, Promise, PromiseError, PromiseOrValue,
};

mod ft;
mod lost_found;
mod native;
mod nft;
mod rollback;

const GAS_FOR_RESOLVE_SWAP: Gas = Gas::from_tgas(5);

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
    ) -> Result<(), SwapError> {
        if create.deadline.has_expired() {
            return Err(SwapError::Expired);
        }
        match self.intents.entry(create.id) {
            Entry::Occupied(entry) => {
                return Err(SwapError::AlreadyExists(entry.key().clone()));
            }
            Entry::Vacant(entry) => {
                entry.insert(
                    SwapIntentStatus::Available(SwapIntent {
                        initiator: sender,
                        asset_in,
                        asset_out: create.asset_out,
                        recipient: create.recipient,
                        deadline: create.deadline,
                    })
                    .into(),
                );
            }
        }
        Ok(())
    }

    fn execute_intent(
        &mut self,
        sender: AccountId,
        received: Asset,
        execute: ExecuteSwapIntentAction,
    ) -> Result<Promise, SwapError> {
        // we remove asset here since there is no need to process
        let intent = self
            .intents
            .get_mut(&execute.id)
            .ok_or_else(|| SwapError::NotFound(execute.id.clone()))?
            .lock()
            .ok_or(SwapError::Locked)?
            .as_available()
            .ok_or(SwapError::WrongStatus)?;

        if intent.has_expired() {
            return Err(SwapError::Expired);
        }
        if received != intent.asset_out {
            return Err(SwapError::WrongAsset);
        }

        // ensure that we have enough gas to transfer both assets
        // TODO: maybe we can omit this check and specify static gas manually,
        // so that the current tx would revert and promises would not be created
        if env::prepaid_gas().saturating_sub(env::used_gas())
            < intent
                .asset_in
                .gas_for_transfer()
                .saturating_add(intent.asset_out.gas_for_transfer())
                // TODO: reserve gas for multiple stages
                .saturating_add(GAS_FOR_RESOLVE_SWAP)
        {
            return Err(SwapError::InsufficientGas);
        }

        // TODO: structured JSON logs
        log!("Intent '{}' fulfilled successfully", execute.id);

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
            Self::ext(env::current_account_id()).resolve_transfer_asset_out(
                &execute.id,
                sender.clone(),
                execute.recipient,
            ),
        ))
    }

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
    #[private]
    pub fn resolve_transfer_asset_out(
        &mut self,
        id: &IntentId,
        asset_out_sender: AccountId,
        #[callback_result] transfer_asset_out: Result<(), PromiseError>,
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
    ) -> Result<PromiseOrValue<serde_json::Value>, SwapError> {
        let intent = self
            .intents
            .get_mut(id)
            .ok_or_else(|| SwapError::NotFound(id.clone()))?;

        if !transfer_asset_out_succeeded {
            let intent = intent
                .unlock()
                .ok_or(SwapError::Unlocked)?
                .as_available()
                .ok_or(SwapError::WrongStatus)?;
            return Ok(match intent.asset_out {
                Asset::Native(amount) => {
                    // TODO: what if it fails?
                    // refund manually
                    Self::transfer_native(amount, asset_out_sender).into()
                    // TODO: return promise that returns bool
                }
                Asset::Ft(FtAmount { amount, .. }) => {
                    // return back to sender
                    PromiseOrValue::Value(json!(U128(amount)))
                }
                Asset::Nft(_) => {
                    // return back to sender
                    PromiseOrValue::Value(json!(true))
                }
            });
        }

        let intent = intent
            .as_locked()
            .ok_or(SwapError::Unlocked)?
            .as_available()
            .ok_or(SwapError::WrongStatus)?;
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
    // TODO: return enum AssetOnTransferOutput
    #[private]
    pub fn resolve_transfer_asset_in(
        &mut self,
        id: &IntentId,
        #[callback_result] transfer_asset_in: Result<(), PromiseError>,
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
    ) -> Result<serde_json::Value, SwapError> {
        let intent = self
            .intents
            .get_mut(id)
            .ok_or_else(|| SwapError::NotFound(id.clone()))?
            .unlock()
            .ok_or(SwapError::Unlocked)?;

        let swap = intent.as_available().ok_or(SwapError::WrongStatus)?;
        let asset_out = swap.asset_out.clone();

        if transfer_asset_in_succeeded {
            self.intents.remove(id);
        } else {
            // TODO: log
            *intent = SwapIntentStatus::Lost(LostAsset {
                asset: swap.asset_in.clone(),
                recipient: asset_in_recipient,
            });
        }

        Ok(match asset_out {
            // close self.native_action(asset_out)
            Asset::Native(_) => json!(true),
            // close ft_on_transfer(asset_out)
            Asset::Ft(_) => json!(U128(0)),
            // close nft_on_transfer(asset_out)
            Asset::Nft(_) => json!(false),
        })
    }
}