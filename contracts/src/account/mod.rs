use near_sdk::{ext_contract, near, AccountId};

pub use self::error::AccountError;

mod error;

// TODO: make this contract an extension of NFT, since every Account
// is a unique item. This will ease the integration of Defuse
// with already existing tooling around NFTs
#[ext_contract(ext_account_contract)]
pub trait AccountContract {
    /// Create an account with given defivation path for given owner
    fn create_account(&mut self, owner: AccountId, derivation_path: String);
    /// Change an owner of account with given derivation path.
    fn change_owner(&mut self, from: &AccountId, to: AccountId, derivation_path: String);
    /// Return all [`Account`]s owned by given `owner`
    fn get_accounts(&self, owner: &AccountId) -> Vec<(String, Account)>;

    /// Return MPC contract for this
    fn mpc_contract(&self) -> &AccountId;
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
#[near(serializers=[borsh, json])]
pub struct Account {
    is_locked: bool,
}
