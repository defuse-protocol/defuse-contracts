use near_contract_standards::non_fungible_token::{
    core::NonFungibleTokenCore, NonFungibleTokenEnumeration, NonFungibleTokenResolver,
};
use near_sdk::{ext_contract, AccountId};

// TODO: make this contract an extension of NFT, since every Account
// is a unique item. This will ease the integration of Defuse
// with already existing tooling around NFTs
#[ext_contract(ext_account_contract)]
pub trait AccountContract:
    NonFungibleTokenCore + NonFungibleTokenResolver + NonFungibleTokenEnumeration
{
    /// Create an account with given defivation path for given owner
    /// By default, owner is sender
    // TODO: maybe accept optional derivation path, so it can be also generated on-chain?
    fn create_account(&mut self, derivation_path: String, owner: Option<AccountId>);
}
