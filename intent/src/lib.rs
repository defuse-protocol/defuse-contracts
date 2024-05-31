use near_sdk::env::panic_str;
use near_sdk::json_types::U128;
use near_sdk::store::{LookupMap, LookupSet};
use near_sdk::{
    env, ext_contract, log, near, AccountId, BorshStorageKey, NearToken, PanicOnDefault, Promise,
    PromiseOrValue,
};

use crate::{types::intent::Action, types::Intent};

pub mod error;
pub mod types;

#[derive(BorshStorageKey)]
#[near(serializers=[borsh])]
enum Prefix {
    SupportedTokens,
    AllowedSolvers,
    Intents,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct IntentContract {
    owner_id: AccountId,
    supported_tokens: LookupSet<String>,
    allowed_solvers: LookupSet<AccountId>,
    intents: LookupMap<String, Intent>,
}

#[near]
impl IntentContract {
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
        }
    }

    /// Add a new solver to the whitelist.
    pub fn add_solver(&mut self, solver_id: AccountId) {
        self.assert_owner();
        self.allowed_solvers.insert(solver_id);
    }

    /// The callback is called by NEP-141 after `ft_transfer_call`.
    ///
    /// # Panics
    ///
    /// The panic occurs if an attempt to add an intent with an existing id or execute
    /// a nonexistent intent.
    pub fn ft_on_transfer(
        &mut self,
        sender_id: &AccountId,
        amount: U128,
        msg: &String,
    ) -> PromiseOrValue<U128> {
        // Validate that sender_id is in white token list.
        // self.assert_token(&sender_id); // TODO: Check if we need tokens validation.
        let action = Action::decode(msg)
            .unwrap_or_else(|e| panic_str(&format!("Action decode error: {}", e.as_ref())));

        log!(format!("{sender_id} : {}: msg: {msg}", amount.0));

        match action {
            Action::CreateIntent((id, intent)) => {
                assert!(
                    self.intents.insert(id, intent).is_none(),
                    "Intent already exists"
                );

                PromiseOrValue::Value(0.into())
            }
            Action::ExecuteIntent(id) => {
                let current_id = env::current_account_id();
                let solver_id = env::signer_account_id();
                self.assert_solver(&solver_id);

                let intent = self
                    .intents
                    .get(&id)
                    .unwrap_or_else(|| panic_str(&format!("No intent for id: {id}")));

                let promise = if intent.is_expired() {
                    Self::ext(current_id).rollback_intent(&id)
                } else {
                    ext_ft::ext(intent.send.token_id.clone())
                        .with_attached_deposit(NearToken::from_yoctonear(1))
                        .ft_transfer(solver_id, intent.send.amount)
                        .then(
                            ext_ft::ext(intent.receive.token_id.clone())
                                .with_attached_deposit(NearToken::from_yoctonear(1))
                                .ft_transfer(intent.initiator.clone(), intent.receive.amount),
                        )
                        .then(Self::ext(current_id).cleanup_intent(&id))
                };

                PromiseOrValue::Promise(promise)
            }
        }
    }

    /// Callback which removes an intent after successful execution.
    #[private]
    pub fn cleanup_intent(&mut self, intent_id: &String) {
        self.intents.remove(intent_id);
    }

    /// Rollback created intent and refund tokens to the intent's initiator.
    /// The transaction could be called by an intent initiator or owner.
    ///
    /// # Panics
    ///
    /// The panic occurs if intent doesn't exist of caller is not allowed.
    pub fn rollback_intent(&mut self, id: &String) -> Promise {
        let intent = self
            .intents
            .get(id)
            .unwrap_or_else(|| panic_str(&format!("No intent for id: {id}")));
        let predecessor_id = env::predecessor_account_id();

        assert!(
            predecessor_id == intent.initiator
                || predecessor_id == self.owner_id
                || predecessor_id == env::current_account_id(),
            "Only initiator, self or owner can roll back the intent"
        );

        ext_ft::ext(intent.send.token_id.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .ft_transfer(intent.initiator.clone(), intent.send.amount)
            .then(Self::ext(env::current_account_id()).cleanup_intent(id))
    }

    /// Set a new owner of the contract.
    pub fn set_owner(&mut self, owner_id: AccountId) {
        self.assert_owner();
        self.owner_id = owner_id;
    }

    /// Return owner of the contract.
    pub const fn get_owner(&self) -> &AccountId {
        &self.owner_id
    }

    /// Return pending intent by id.
    pub fn get_intent(&self, id: &String) -> Option<&Intent> {
        self.intents.get(id)
    }

    /// Check if the provided solver is allowed.
    pub fn is_allowed_solver(&self, solver_id: &AccountId) -> bool {
        self.allowed_solvers.contains(solver_id)
    }

    fn assert_owner(&self) {
        assert_eq!(
            self.owner_id,
            env::predecessor_account_id(),
            "Only owner is allowed to add a new solver"
        );
    }

    fn assert_solver(&self, solver_id: &AccountId) {
        assert!(
            self.allowed_solvers.contains(solver_id),
            "The solver is not allowed"
        );
    }

    #[allow(dead_code)]
    fn assert_token(&self, token_id: &AccountId) {
        assert!(
            self.supported_tokens.contains(token_id.as_str()),
            "Unsupported token"
        );
    }
}

#[ext_contract(ext_ft)]
pub trait FungibleToken {
    fn ft_balance_of(&self, account_id: AccountId) -> U128;

    fn ft_transfer(&self, receiver_id: AccountId, amount: U128);

    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128>;
}
