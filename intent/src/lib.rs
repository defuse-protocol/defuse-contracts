use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::store::{LookupMap, LookupSet};
use near_sdk::{
    assert_one_yocto, env, ext_contract, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault,
    PromiseOrValue,
};

use crate::types::{Intent, IntentType, Status};

pub mod types;

#[derive(BorshSerialize, BorshDeserialize, BorshStorageKey)]
#[borsh(crate = "near_sdk::borsh")]
enum Prefix {
    SupportedTokens,
    AllowedSolvers,
    Intents,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[borsh(crate = "near_sdk::borsh")]
pub struct IntentContract {
    owner_id: AccountId,
    supported_tokens: LookupSet<String>,
    allowed_solvers: LookupSet<AccountId>,
    intents: LookupMap<String, Intent>,
}

#[near_bindgen]
impl IntentContract {
    /// Contract constructor.
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            supported_tokens: LookupSet::new(Prefix::SupportedTokens),
            allowed_solvers: LookupSet::new(Prefix::AllowedSolvers),
            intents: LookupMap::new(Prefix::Intents),
        }
    }

    /// Create a new intent.
    #[payable]
    pub fn create_intent(&mut self, id: String, intent: IntentType) {
        assert_one_yocto();

        let intent_owner = env::signer_account_id();
        let intent = Intent::new(intent_owner, intent);

        self.intents.insert(id, intent);
    }

    /// Execute an intent. The transaction could be called by allowed solvers only.
    pub fn execute_intent(&mut self, intent_id: String) {
        // check that caller(solver) is in allowed list.
        let solver_id = env::predecessor_account_id();

        assert!(
            self.allowed_solvers.contains(&solver_id),
            "The caller is not allowed"
        );

        let intent = self.intents.get_mut(&intent_id).expect("No such intent");

        match intent.intent_type {
            IntentType::Nep141(_) => execute_nep141_intent(solver_id, intent),
        }
    }

    /// Add a new solver to the whitelist.
    pub fn add_solver(&mut self, solver_id: AccountId) {
        self.assert_owner();
        self.allowed_solvers.insert(solver_id);
    }

    fn assert_owner(&self) {
        assert_eq!(
            self.owner_id,
            env::predecessor_account_id(),
            "Only owner allowed to add a new solver"
        );
    }
}

fn execute_nep141_intent(_solver_id: AccountId, intent: &mut Intent) {
    assert_eq!(intent.status, Status::CreatedByUser, "Bad intent status");

    intent.status = Status::ApprovedBySolver;
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
