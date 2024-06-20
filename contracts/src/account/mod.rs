use near_contract_standards::non_fungible_token::{
    core::NonFungibleTokenCore, NonFungibleTokenEnumeration,
};
use near_sdk::{ext_contract, near, AccountId};

pub use self::error::AccountError;

mod error;

// TODO: make this contract an extension of NFT, since every Account
// is a unique item. This will ease the integration of Defuse
// with already existing tooling around NFTs
#[ext_contract(ext_account_contract)]
pub trait AccountContract: NonFungibleTokenCore + NonFungibleTokenEnumeration {
    /// Create an account with given defivation path for given owner
    // TODO: maybe accept optional derivation path, so it can be also generated on-chain?
    fn create_account(&mut self, owner: AccountId, derivation_path: String);

    /// Return MPC contract for this
    fn mpc_contract(&self) -> &AccountId;
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
#[near(serializers=[borsh, json])]
pub struct Account {
    is_locked: bool,
}
