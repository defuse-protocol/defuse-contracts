use defuse_contracts::account::{Account, AccountContract};
use near_sdk::{env, near, store::LookupSet, AccountId, BorshStorageKey, PanicOnDefault};

use self::account_db::AccountDb;

mod account_db;

#[derive(BorshStorageKey)]
#[near(serializers=[borsh])]
enum Prefix {
    Accounts,
    Indexers,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct AccountContractImpl {
    owner_id: AccountId,
    /// MPC contract id.
    mpc_contract_id: AccountId,
    /// List of indexers. Accounts which allow to add a new account.
    indexers: LookupSet<AccountId>,
    /// Key here is `account_id + '.' + derivation_path`
    /// Value is account id of the owner.
    accounts: AccountDb,
}

#[near]
impl AccountContract for AccountContractImpl {
    fn create_account(&mut self, account_id: AccountId, derivation_path: String) {
        // Only indexers can call this transaction.
        let predecessor_id = env::predecessor_account_id();
        self.assert_indexer(&predecessor_id);

        self.accounts
            .add_account(account_id, derivation_path, Account::default())
            .unwrap();
    }

    fn change_owner(&mut self, from: &AccountId, to: AccountId, derivation_path: String) {
        self.accounts
            .change_owner(from, to, derivation_path)
            .unwrap();
    }

    fn get_accounts(&self, account_id: &AccountId) -> Vec<(String, Account)> {
        self.accounts.get_accounts(account_id).unwrap()
    }

    fn mpc_contract(&self) -> &AccountId {
        &self.mpc_contract_id
    }
}

#[near]
impl AccountContractImpl {
    #[init]
    #[must_use]
    #[allow(clippy::use_self)]
    pub fn new(owner_id: AccountId, mpc_contract_id: AccountId) -> Self {
        Self {
            owner_id,
            mpc_contract_id,
            indexers: LookupSet::new(Prefix::Indexers),
            accounts: AccountDb::new(Prefix::Accounts),
        }
    }

    #[private]
    pub fn set_mpc_contract(&mut self, contract_id: AccountId) {
        self.mpc_contract_id = contract_id;
    }

    fn assert_indexer(&self, account_id: &AccountId) {
        assert!(
            self.indexers.contains(account_id),
            "Only indexers allow adding an account"
        );
    }
}
