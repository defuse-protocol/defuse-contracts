use defuse_contracts::{
    intents::swap::{
        Asset, CreateSwapIntentAction, FtAmount, FulfillSwapIntentAction, IntentId, Lost, Swap,
        SwapError, SwapIntent, SwapIntentContract,
    },
    utils::Mutex,
};

use near_sdk::{
    env,
    json_types::U128,
    log, near,
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
    intents: LookupMap<IntentId, Mutex<SwapIntent>>,
}

#[derive(BorshStorageKey)]
#[near(serializers = [borsh])]
enum Prefix {
    Intents,
}

#[near]
impl SwapIntentContractImpl {
    #[init]
    pub fn new() -> Self {
        Self {
            intents: LookupMap::new(Prefix::Intents),
        }
    }
}

#[near]
impl SwapIntentContract for SwapIntentContractImpl {
    fn get_swap_intent(&self, id: &IntentId) -> Option<&Mutex<SwapIntent>> {
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
                    SwapIntent::Swap(Swap {
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

    fn fulfill_intent(
        &mut self,
        sender: AccountId,
        received: Asset,
        fulfill: FulfillSwapIntentAction,
    ) -> Result<Promise, SwapError> {
        // we remove asset here since there is no need to process
        let intent = self
            .intents
            .get_mut(&fulfill.id)
            .ok_or_else(|| SwapError::NotFound(fulfill.id.clone()))?
            .lock_mut()
            .ok_or(SwapError::Locked)?
            .as_swap()
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
        log!("Intent '{}' fulfilled successfully", fulfill.id);

        Ok(Self::transfer(
            &fulfill.id,
            intent.asset_out.clone(),
            intent
                .recipient
                .as_ref()
                .unwrap_or(&intent.initiator)
                .clone(),
        )
        .then(
            Self::ext(env::current_account_id()).resolve_transfer_asset_out(
                &fulfill.id,
                sender.clone(),
                // TODO: pass option, unwrap_or() inside this call
                fulfill.recipient.unwrap_or(sender),
            ),
        ))
    }

    #[inline]
    fn transfer(id: &IntentId, asset: Asset, recipient: AccountId) -> Promise {
        match asset {
            Asset::Native(amount) => Self::transfer_native(amount, recipient),
            Asset::Ft(ft) => Self::transfer_ft(ft, recipient, format!("{id}")),
            Asset::Nft(nft) => Self::transfer_nft(nft, recipient, format!("{id}")),
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
        asset_in_recipient: AccountId,
    ) -> PromiseOrValue<serde_json::Value> {
        let intent = self
            .intents
            .get_mut(id)
            .ok_or_else(|| SwapError::NotFound(id.clone()))
            .unwrap();

        if transfer_asset_out.is_err() {
            let intent = intent
                .unlock()
                .ok_or(SwapError::Unlocked)
                .unwrap()
                .as_swap()
                .ok_or(SwapError::WrongStatus)
                .unwrap();
            return match intent.asset_out {
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
            };
        }

        let intent = intent
            .get_locked()
            .ok_or(SwapError::Unlocked)
            .unwrap()
            .as_swap()
            .ok_or(SwapError::WrongStatus)
            .unwrap();
        Self::transfer(id, intent.asset_in.clone(), asset_in_recipient.clone())
            .then(
                Self::ext(env::current_account_id())
                    .resolve_transfer_asset_in(id, asset_in_recipient),
            )
            .into()
    }

    #[private]
    pub fn resolve_transfer_asset_in(
        &mut self,
        id: &IntentId,
        #[callback_result] transfer_asset_in: Result<(), PromiseError>,
        asset_in_recipient: AccountId,
    ) -> serde_json::Value {
        let intent = self
            .intents
            .get_mut(id)
            .ok_or_else(|| SwapError::NotFound(id.clone()))
            .unwrap()
            .unlock_mut()
            .ok_or(SwapError::Unlocked)
            .unwrap();

        let swap = intent
            .as_swap()
            .ok_or(SwapError::WrongStatus)
            .unwrap()
            .clone();

        if transfer_asset_in.is_err() {
            // TODO: log
            *intent = SwapIntent::Lost(Lost {
                asset: swap.asset_in.clone(),
                recipient: asset_in_recipient,
            });
        } else {
            self.intents.remove(id);
        }

        match swap.asset_out {
            // close self.native_action(asset_out)
            Asset::Native(_) => json!(true),
            // close ft_on_transfer(asset_out)
            Asset::Ft(_) => json!(U128(0)),
            // close nft_on_transfer(asset_out)
            Asset::Nft(_) => json!(false),
        }
    }
}
