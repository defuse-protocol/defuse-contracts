use defuse_contracts::intent::{
    Action, DetailedIntent, Intent, IntentContract, IntentError, Status,
};

use near_contract_standards::fungible_token::{core::ext_ft_core, receiver::FungibleTokenReceiver};
use near_contract_standards::storage_management::{ext_storage_management, StorageBalance};
use near_gas::NearGas;
use near_sdk::{
    env,
    json_types::U128,
    log, near, require,
    store::{
        lookup_map::{Entry, LookupMap},
        LookupSet,
    },
    AccountId, BorshStorageKey, NearToken, PanicOnDefault, Promise, PromiseOrValue,
};

const DEFAULT_MIN_TTL: u64 = 60; // 1 minute

// Gas
const FINISH_CREATING_GAS: NearGas = NearGas::from_tgas(5);
const FINISH_EXECUTING_GAS: NearGas = NearGas::from_tgas(20);
const ROLLBACK_INTENT_GAS: NearGas = NearGas::from_tgas(10);

#[derive(BorshStorageKey)]
#[near(serializers=[borsh])]
enum Prefix {
    SupportedTokens,
    AllowedSolvers,
    Intents,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct IntentContractImpl {
    owner_id: AccountId,
    supported_tokens: LookupSet<String>,
    allowed_solvers: LookupSet<AccountId>,
    intents: LookupMap<String, DetailedIntent>,
    min_intent_ttl: u64,
}

#[near]
impl IntentContract for IntentContractImpl {
    /// Add a new solver to the whitelist.
    fn add_solver(&mut self, solver_id: AccountId) {
        self.assert_owner();
        self.allowed_solvers.insert(solver_id);
    }

    fn rollback_intent(&mut self, id: String) -> Promise {
        let detailed_intent = self
            .intents
            .get_mut(&id)
            .ok_or_else(|| IntentError::NotFound(id.clone()))
            .unwrap();

        if !detailed_intent.could_be_rollbacked() {
            env::panic_str("Too early to roll back the intent");
        }
        require!(
            matches!(detailed_intent.status(), Status::Available),
            "Only intents with created status could be rolled back"
        );
        detailed_intent.set_status(Status::Processing);

        let predecessor_id = env::predecessor_account_id();
        let intent = detailed_intent.intent();

        require!(
            predecessor_id == intent.initiator
                || predecessor_id == self.owner_id
                || predecessor_id == env::current_account_id(),
            "Only initiator, self or owner can roll back the intent"
        );

        require!(
            env::prepaid_gas().saturating_sub(env::used_gas()) >= ROLLBACK_INTENT_GAS,
            "Not enough gas to rollback the intent"
        );

        ext_ft_core::ext(intent.send.token_id.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .ft_transfer(intent.initiator.clone(), intent.send.amount, None)
            .then(Self::ext(env::current_account_id()).change_intent_status(
                &id,
                Status::RolledBack,
                0.into(),
            ))
    }

    fn get_intent(&self, id: String) -> Option<&DetailedIntent> {
        self.intents.get(&id)
    }

    fn is_allowed_solver(&self, solver_id: AccountId) -> bool {
        self.allowed_solvers.contains(&solver_id)
    }
}

#[near]
impl FungibleTokenReceiver for IntentContractImpl {
    /// The callback is called by NEP-141 after `ft_transfer_call`.
    ///
    /// # Panics
    ///
    /// The panic occurs if an attempt to add an intent with an existing id or execute
    /// a nonexistent intent.
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        // Validate that sender_id is in white token list.
        // self.assert_token(&sender_id); // TODO: Check if we need tokens validation.
        let action = Action::decode(msg).expect("decode Action");

        let promise = match action {
            Action::CreateIntent(id, intent) => {
                log!(
                    "Creating the intent with id: {id} by: {sender_id}, amount: {}",
                    amount.0
                );

                require!(id.len() <= 128, "ID is too long");

                // First check that initiator has storage deposit on token he wants to get.
                ext_storage_management::ext(intent.receive.token_id.clone())
                    .storage_balance_of(sender_id)
                    .then(
                        Self::ext(env::current_account_id())
                            .with_static_gas(FINISH_CREATING_GAS)
                            .finish_creating_intent(id, amount, intent),
                    )
            }
            Action::ExecuteIntent(id) => {
                log!(
                    "Executing the intent with id: {id} by: {sender_id}, amount: {}",
                    amount.0
                );
                let detailed_intent = self
                    .intents
                    .get(&id)
                    .ok_or_else(|| IntentError::NotFound(id.clone()))
                    .unwrap();

                // First check that the solver has storage deposit on token he wants to get.
                ext_storage_management::ext(detailed_intent.intent().send.token_id.clone())
                    .storage_balance_of(sender_id.clone())
                    .then(
                        Self::ext(env::current_account_id())
                            .with_static_gas(FINISH_EXECUTING_GAS)
                            .finish_executing_intent(&id, amount, &sender_id),
                    )
            }
        };

        PromiseOrValue::Promise(promise)
    }
}

#[near]
impl IntentContractImpl {
    /// Contract constructor.
    #[init]
    #[must_use]
    #[allow(clippy::use_self)]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            supported_tokens: LookupSet::new(Prefix::SupportedTokens),
            allowed_solvers: LookupSet::new(Prefix::AllowedSolvers),
            intents: LookupMap::new(Prefix::Intents),
            min_intent_ttl: DEFAULT_MIN_TTL,
        }
    }

    /// Callback which changes a status of the intent. The `amount` arg makes sense in case of
    /// calling this transaction by `ft_transfer_call`. In case if the intent is expired the amount
    /// should be the same as the solver send to return the funds back. In successful execution the
    /// amount should be 0 to prevent refund funds in the `ft_resolve_transfer` callback to the solver.
    ///
    /// # Panics
    ///
    /// Panics if intent with given ID doesn't exist.
    #[private]
    pub fn change_intent_status(
        &mut self,
        intent_id: &String,
        status: Status,
        amount: U128,
    ) -> PromiseOrValue<U128> {
        log!(
            "Changing status of the intent with id: {} to {status:?} status",
            intent_id
        );
        let intent_with_status = self
            .intents
            .get_mut(intent_id)
            .ok_or_else(|| IntentError::NotFound(intent_id.clone()))
            .unwrap();

        intent_with_status.set_status(status);
        PromiseOrValue::Value(amount)
    }

    /// Callback which finishes creating an intent.
    ///
    /// # Panics
    ///
    /// Panics if the storage deposit is too low or there is no storage deposit for the initiator.
    #[private]
    pub fn finish_creating_intent(
        &mut self,
        id: String,
        amount: U128,
        intent: Intent,
        #[callback_result] result: Result<Option<StorageBalance>, near_sdk::PromiseError>,
    ) -> PromiseOrValue<U128> {
        match result {
            Ok(Some(_)) => self.create_intent(id, amount, intent).unwrap(),
            Ok(None) => env::panic_str(&format!("No storage deposit for: {}", &intent.initiator)),
            Err(e) => env::panic_str(&format!("Error getting storage deposit: {e:?}")),
        }
    }

    /// Callback which finishes executing an intent.
    ///
    /// # Panics
    ///
    /// Panics if the storage deposit is too low or there is no storage deposit for the solver.
    #[private]
    pub fn finish_executing_intent(
        &mut self,
        id: &String,
        amount: U128,
        solver_id: &AccountId,
        #[callback_result] result: Result<Option<StorageBalance>, near_sdk::PromiseError>,
    ) -> PromiseOrValue<U128> {
        match result {
            Ok(Some(_)) => self
                .execute_intent(id, amount)
                .unwrap_or_else(|e| env::panic_str(&e.to_string())),
            Ok(None) => env::panic_str(&format!("No storage deposit for: {solver_id}")),
            Err(e) => env::panic_str(&format!("Error getting storage deposit: {e:?}")),
        }
    }

    /// Set a new owner of the contract.
    #[inline]
    pub fn set_owner(&mut self, owner_id: AccountId) {
        self.assert_owner();
        self.owner_id = owner_id;
    }

    /// Return owner of the contract.
    #[inline]
    pub const fn get_owner(&self) -> &AccountId {
        &self.owner_id
    }

    /// Set the minimum TTL for the intent.
    ///
    /// # Panics
    ///
    /// A panic could be thrown if the provided TTL is too long
    /// or the transaction is invoked not by the owner.
    pub fn set_min_intent_ttl(&mut self, min_ttl: u64) {
        self.assert_owner();
        // Check for too long value of TTL
        require!(min_ttl.checked_mul(1000).is_some(), "TTL is too long");
        self.min_intent_ttl = min_ttl;
    }

    /// Return the minimum time to live for the intent.
    pub const fn get_min_intent_ttl(&self) -> u64 {
        self.min_intent_ttl
    }

    fn assert_owner(&self) {
        require!(
            self.owner_id == env::predecessor_account_id(),
            "Only owner is allowed to add a new solver"
        );
    }

    #[inline]
    fn assert_solver(&self, solver_id: &AccountId) {
        require!(
            self.allowed_solvers.contains(solver_id),
            "The solver is not allowed"
        );
    }

    #[allow(dead_code)]
    #[inline]
    fn assert_token(&self, token_id: &AccountId) {
        require!(
            self.supported_tokens.contains(token_id.as_str()),
            "Unsupported token"
        );
    }

    fn create_intent(
        &mut self,
        id: String,
        amount: U128,
        intent: Intent,
    ) -> Result<PromiseOrValue<U128>, IntentError> {
        if amount != intent.send.amount {
            return Err(IntentError::AmountMismatch);
        }

        match self.intents.entry(id) {
            Entry::Occupied(entry) => Err(IntentError::AlreadyExists(entry.key().clone())),
            Entry::Vacant(entry) => {
                let detailed_intent = DetailedIntent::new(intent, self.min_intent_ttl);
                entry.insert(detailed_intent);

                Ok(PromiseOrValue::Value(0.into()))
            }
        }
    }

    fn execute_intent(
        &mut self,
        id: &String,
        amount: U128,
    ) -> Result<PromiseOrValue<U128>, IntentError> {
        let solver_id = env::signer_account_id();

        log!(
            "Executing the intent with id: {id} by: {}, amount: {}",
            &solver_id,
            amount.0
        );

        self.assert_solver(&solver_id);

        let detailed_intent = self
            .intents
            .get_mut(id)
            .ok_or_else(|| IntentError::NotFound(id.clone()))
            .unwrap();

        if !matches!(detailed_intent.status(), Status::Available) {
            return Err(IntentError::WrongStatus);
        }

        detailed_intent.set_status(Status::Processing);

        let intent = detailed_intent.intent();
        let current_id = env::current_account_id();

        let promise = if detailed_intent.intent().is_expired() {
            ext_ft_core::ext(intent.send.token_id.clone())
                .with_attached_deposit(NearToken::from_yoctonear(1))
                .ft_transfer(intent.initiator.clone(), intent.send.amount, None)
                .then(Self::ext(current_id).change_intent_status(id, Status::Expired, amount))
        } else {
            if amount != intent.receive.amount {
                return Err(IntentError::AmountMismatch);
            }

            ext_ft_core::ext(intent.send.token_id.clone())
                .with_attached_deposit(NearToken::from_yoctonear(1))
                .ft_transfer(solver_id, intent.send.amount, None)
                .and(
                    ext_ft_core::ext(intent.receive.token_id.clone())
                        .with_attached_deposit(NearToken::from_yoctonear(1))
                        .ft_transfer(intent.initiator.clone(), intent.receive.amount, None),
                )
                .then(Self::ext(current_id).change_intent_status(id, Status::Completed, 0.into()))
        };

        Ok(PromiseOrValue::Promise(promise))
    }
}
