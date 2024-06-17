use defuse_contracts::intent::{
    Action, DetailedIntent, Intent, IntentContract, IntentError, Status,
};

use near_contract_standards::fungible_token::{core::ext_ft_core, receiver::FungibleTokenReceiver};
use near_sdk::{
    env,
    json_types::U128,
    log, near,
    store::{
        lookup_map::{Entry, LookupMap},
        LookupSet,
    },
    AccountId, BorshStorageKey, NearToken, PanicOnDefault, Promise, PromiseOrValue,
};

const DEFAULT_MIN_TTL: u64 = 60; // 1 minute

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
        assert!(
            matches!(detailed_intent.status(), Status::Available),
            "Only intents with created status could be rolled back"
        );
        detailed_intent.set_status(Status::Processing);

        let predecessor_id = env::predecessor_account_id();
        let intent = detailed_intent.intent();

        assert!(
            predecessor_id == intent.initiator
                || predecessor_id == self.owner_id
                || predecessor_id == env::current_account_id(),
            "Only initiator, self or owner can roll back the intent"
        );

        ext_ft_core::ext(intent.send.token_id.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .ft_transfer(intent.initiator.clone(), intent.send.amount, None)
            .then(
                Self::ext(env::current_account_id()).change_intent_status(&id, Status::RolledBack),
            )
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

        match action {
            Action::CreateIntent(id, intent) => {
                log!(
                    "Creating the intent with id: {id} by: {sender_id}, amount: {}",
                    amount.0
                );
                self.create_intent(id, amount, intent).unwrap()
            }
            Action::ExecuteIntent(id) => self.execute_intent(&id, amount).unwrap(),
        }
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

    /// Callback which changes a status of the intent.
    ///
    /// # Panics
    ///
    /// Panics if intent with given ID doesn't exist.
    #[private]
    pub fn change_intent_status(&mut self, intent_id: &String, status: Status) {
        let intent_with_status = self
            .intents
            .get_mut(intent_id)
            .ok_or_else(|| IntentError::NotFound(intent_id.clone()))
            .unwrap();

        intent_with_status.set_status(status);
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
        assert!(min_ttl.checked_mul(1000).is_some(), "TTL is too long");
        self.min_intent_ttl = min_ttl;
    }

    /// Return the minimum time to live for the intent.
    pub const fn get_min_intent_ttl(&self) -> u64 {
        self.min_intent_ttl
    }

    fn assert_owner(&self) {
        assert_eq!(
            self.owner_id,
            env::predecessor_account_id(),
            "Only owner is allowed to add a new solver"
        );
    }

    #[inline]
    fn assert_solver(&self, solver_id: &AccountId) {
        assert!(
            self.allowed_solvers.contains(solver_id),
            "The solver is not allowed"
        );
    }

    #[allow(dead_code)]
    #[inline]
    fn assert_token(&self, token_id: &AccountId) {
        assert!(
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
            .ok_or_else(|| IntentError::NotFound(id.clone()))?;

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
                .then(Self::ext(current_id).change_intent_status(id, Status::Expired))
        } else {
            if amount != intent.receive.amount {
                return Err(IntentError::AmountMismatch);
            }

            ext_ft_core::ext(intent.send.token_id.clone())
                .with_attached_deposit(NearToken::from_yoctonear(1))
                .ft_transfer(solver_id, intent.send.amount, None)
                .then(
                    ext_ft_core::ext(intent.receive.token_id.clone())
                        .with_attached_deposit(NearToken::from_yoctonear(1))
                        .ft_transfer(intent.initiator.clone(), intent.receive.amount, None),
                )
                .then(Self::ext(current_id).change_intent_status(id, Status::Completed))
        };

        Ok(PromiseOrValue::Promise(promise))
    }
}
