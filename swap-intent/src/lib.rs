use defuse_contracts::intents::swap::{
    Asset, CreateSwapIntentAction, FulfillSwapIntentAction, IntentID, SwapError, SwapIntent,
    SwapIntentAction, SwapIntentContract,
};

use near_sdk::{
    env, log, near,
    store::lookup_map::{Entry, LookupMap},
    AccountId, BorshStorageKey, NearToken, PanicOnDefault, Promise, PromiseError, PromiseOrValue,
};

mod ft;
mod nft;

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct SwapIntentContractImpl {
    intents: LookupMap<IntentID, SwapIntent>,
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

    #[private]
    #[allow(unused_variables)] // TODO
    pub fn resolve_swap(
        &mut self,
        id: IntentID,
        #[callback_result] transfer_to_user: Result<(), PromiseError>,
        #[callback_result] transfer_to_solver: Result<(), PromiseError>,
    ) {
        // TODO: handle failed transfers
        // It can be implemented using lost&found approach from ref.finance:
        // https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/account_deposit.rs#L435
    }
}

#[near]
impl SwapIntentContract for SwapIntentContractImpl {
    fn get_swap_intent(&self, id: &IntentID) -> Option<&SwapIntent> {
        self.intents.get(id)
    }

    #[payable]
    fn native_action(&mut self, action: SwapIntentAction) -> PromiseOrValue<()> {
        let amount = env::attached_deposit();
        assert!(!amount.is_zero());
        self.handle_action(env::predecessor_account_id(), Asset::Native(amount), action)
            .unwrap()
    }

    fn rollback_intent(&mut self, id: IntentID) -> Promise {
        let intent = self
            .intents
            .remove(&id)
            .ok_or_else(|| SwapError::NotFound(id.clone()))
            .unwrap();
        assert!(
            env::prepaid_gas().saturating_sub(env::used_gas())
                >= intent.asset_in.gas_for_transfer()
        );
        // TODO: emit log
        Self::transfer(&id, intent.asset_in, intent.initiator)
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
                    SwapIntent {
                        initiator: sender,
                        asset_in,
                        asset_out: create.asset_out,
                        recipient: create.recipient,
                        deadline: create.deadline,
                    }
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
            .remove(&fulfill.id)
            .ok_or_else(|| SwapError::NotFound(fulfill.id.clone()))?;

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
        {
            return Err(SwapError::InsufficientGas);
        }

        // TODO: structured JSON logs
        log!("Intent '{}' fulfilled successfully", fulfill.id);

        Ok(
            // transfer to user
            Self::transfer(
                &fulfill.id,
                intent.asset_out,
                intent.recipient.unwrap_or(intent.initiator),
            )
            // transfer to solver
            .and(Self::transfer(
                &fulfill.id,
                intent.asset_in,
                fulfill.recipient.unwrap_or(sender),
            ))
            // resolve swap
            .then(Self::ext(env::current_account_id()).resolve_swap(fulfill.id)),
        )
    }

    #[inline]
    fn transfer(id: &IntentID, asset: Asset, recipient: AccountId) -> Promise {
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
