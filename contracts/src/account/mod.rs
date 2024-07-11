use near_contract_standards::non_fungible_token::{
    core::NonFungibleTokenCore, NonFungibleTokenEnumeration, NonFungibleTokenResolver,
};
use near_sdk::{ext_contract, AccountId};

#[ext_contract(ext_account_contract)]
pub trait AccountContract:
    NonFungibleTokenCore + NonFungibleTokenResolver + NonFungibleTokenEnumeration
{
    /// Create an account with given defivation path for given owner
    /// By default, owner is sender
    // TODO: maybe accept optional derivation path, so it can be also generated on-chain?
    // TODO: derivation_path can contain owner_id, so it can be generated
    // off-chain and registration can not be front-runned
    fn create_account(&mut self, derivation_path: String, owner: Option<AccountId>);
}
