use defuse_contracts::{
    intents::swap::{
        Asset, CreateSwapIntentAction, FtAmount, FulfillSwapIntentAction, IntentId, LostFound,
        Swap, SwapError, SwapIntent, SwapIntentAction, SwapIntentContract,
    },
    utils::Mutex,
};

use near_sdk::{
    env,
    json_types::U128,
    log, near,
    serde_json::{self, json},
    store::lookup_map::{Entry, LookupMap},
    AccountId, BorshStorageKey, Gas, NearToken, PanicOnDefault, Promise, PromiseError,
    PromiseOrValue,
};

mod ft;
mod nft;

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
    LostFound,
}

#[near]
impl SwapIntentContractImpl {
    #[init]
    pub fn new() -> Self {
        Self {
            intents: LookupMap::new(Prefix::Intents),
        }
    }

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
        // TODO: assert_locked
        if transfer_asset_out.is_err() {
            let intent = intent
                .unlock_mut()
                .ok_or(SwapError::Unlocked)
                .unwrap()
                .as_swap()
                .ok_or(SwapError::WrongStatus)
                .unwrap();
            return match intent.asset_out {
                Asset::Native(amount) => {
                    // TODO: unlock now?
                    // refund manually
                    Self::transfer_native(amount, asset_out_sender).into()
                }
                Asset::Ft(FtAmount { amount, .. }) => {
                    // TODO: unlock now?
                    // return back to sender
                    PromiseOrValue::Value(json!(U128(amount)))
                }
                Asset::Nft(_) => {
                    // TODO: unlock now?
                    // return back to sender
                    PromiseOrValue::Value(json!(true))
                }
            };
        }
        let intent = unsafe { intent.get_unchecked() }
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
    pub const fn resolve_refund_asset_out() -> bool {
        false
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
            *intent = SwapIntent::LostFound(LostFound {
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

    #[private]
    pub fn resolve_rollback_intent(
        &mut self,
        id: &IntentId,
        #[callback_result] transfer_asset_in: Result<(), PromiseError>,
    ) -> bool {
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

        if transfer_asset_in.is_ok() {
            self.intents.remove(id);
            return true;
        }

        // TODO: log
        *intent = SwapIntent::LostFound(LostFound {
            asset: swap.asset_in.clone(),
            recipient: swap.initiator.clone(),
        });
        false
    }

    #[private]
    pub fn resolve_lost_found(
        &mut self,
        id: &IntentId,
        #[callback_result] transfer: Result<(), PromiseError>,
    ) -> bool {
        let intent = self
            .intents
            .get_mut(id)
            .ok_or_else(|| SwapError::NotFound(id.clone()))
            .unwrap();
        intent
            .unlock_mut()
            .ok_or(SwapError::Unlocked)
            .unwrap()
            .as_lost_found()
            .ok_or(SwapError::WrongStatus)
            .unwrap();

        if transfer.is_ok() {
            self.intents.remove(id);
            return true;
        }

        false
    }
}

#[near]
impl SwapIntentContract for SwapIntentContractImpl {
    fn get_swap_intent(&self, id: &IntentId) -> Option<&Mutex<SwapIntent>> {
        self.intents.get(id)
    }

    #[payable]
    fn native_action(&mut self, action: SwapIntentAction) -> PromiseOrValue<()> {
        let amount = env::attached_deposit();
        assert!(!amount.is_zero());
        self.handle_action(env::predecessor_account_id(), Asset::Native(amount), action)
            .unwrap()
        // TODO: refund if error? or it happens automatically if function_call fails?
    }

    fn rollback_intent(&mut self, id: IntentId) -> PromiseOrValue<bool> {
        let intent = self
            .intents
            .get_mut(&id)
            .ok_or_else(|| SwapError::NotFound(id.clone()))
            .unwrap()
            .lock_mut()
            .ok_or(SwapError::Locked)
            .unwrap()
            .as_swap()
            .ok_or(SwapError::WrongStatus)
            .unwrap();

        // TODO: only initiator

        assert!(
            env::prepaid_gas().saturating_sub(env::used_gas())
                >= intent.asset_in.gas_for_transfer()
        );
        // TODO: emit log
        Self::transfer(&id, intent.asset_in.clone(), intent.initiator.clone())
            .then(Self::ext(env::current_account_id()).resolve_rollback_intent(&id))
            .into()
    }

    fn lost_found(&mut self, id: &IntentId) -> Promise {
        let LostFound { asset, recipient } = self
            .intents
            .get_mut(id)
            .ok_or_else(|| SwapError::NotFound(id.clone()))
            .unwrap()
            .lock_mut()
            .ok_or(SwapError::Locked)
            .unwrap()
            .as_lost_found()
            .ok_or(SwapError::WrongStatus)
            .unwrap();

        Self::transfer(id, asset.clone(), recipient.clone())
            .then(Self::ext(env::current_account_id()).resolve_lost_found(id))
    }
}

impl SwapIntentContractImpl {
    fn handle_action(
        &mut self,
        sender: AccountId,
        received: Asset,
        action: SwapIntentAction,
    ) -> Result<PromiseOrValue<()>, SwapError> {
        match action {
            SwapIntentAction::Create(create) => self
                .create_intent(sender, received, create)
                .map(PromiseOrValue::Value),
            SwapIntentAction::Fulfill(fulfill) => self
                .fulfill_intent(sender, received, fulfill)
                .map(Into::into),
        }
    }

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
                .saturating_add(GAS_FOR_RESOLVE_SWAP)
        {
            return Err(SwapError::InsufficientGas);
        }

        // TODO: structured JSON logs
        log!("Intent '{}' fulfilled successfully", fulfill.id);

        let self_id = env::current_account_id();

        Ok(Self::transfer(
            &fulfill.id,
            intent.asset_out.clone(),
            intent
                .recipient
                .as_ref()
                .unwrap_or(&intent.initiator)
                .clone(),
        )
        .then(Self::ext(self_id).resolve_transfer_asset_out(
            &fulfill.id,
            sender.clone(),
            // TODO: pass option, unwrap_or() inside this call
            fulfill.recipient.unwrap_or(sender),
        )))

        // Ok(
        //     // transfer to solver
        //     Self::transfer(
        //         &fulfill.id,
        //         intent.asset_in,
        //         fulfill.recipient.unwrap_or(sender),
        //     )
        //     // transfer to user
        //     .and(Self::transfer(
        //         &fulfill.id,
        //         intent.asset_out,
        //         intent.recipient.unwrap_or(intent.initiator),
        //     ))
        //     // resolve swap
        //     .then(
        //         Self::ext(env::current_account_id())
        //             .with_static_gas(GAS_FOR_RESOLVE_SWAP)
        //             .resolve_swap(fulfill.id),
        //     ),
        // )
    }

    #[inline]
    fn transfer(id: &IntentId, asset: Asset, recipient: AccountId) -> Promise {
        match asset {
            Asset::Native(amount) => Self::transfer_native(amount, recipient),
            Asset::Ft(ft) => Self::transfer_ft(ft, recipient, format!("{id}")),
            Asset::Nft(nft) => Self::transfer_nft(nft, recipient, format!("{id}")),
        }
    }

    #[inline]
    fn transfer_native(amount: NearToken, recipient: AccountId) -> Promise {
        // TODO: extend with optional function name and args
        // for function_call() to allow further communication
        // with other protocols
        Promise::new(recipient).transfer(amount)
    }
}
