use near_contract_standards::{
    fungible_token::{
        metadata::FungibleTokenMetadataProvider, FungibleTokenCore, FungibleTokenResolver,
    },
    storage_management::StorageManagement,
};
use near_sdk::{ext_contract, json_types::U128, AccountId};

pub const WITHDRAW_MEMO_PREFIX: &str = "WITHDRAW_TO:";

// TODO: maybe handle ft_on_transfer inside ft_transfer_call?
#[ext_contract(ext_poa_fungible_token)]
pub trait POAFungibleToken:
    FungibleTokenCore + FungibleTokenResolver + FungibleTokenMetadataProvider + StorageManagement
{
    fn ft_mint(&mut self, owner_id: AccountId, amount: U128, memo: Option<String>);
}

pub struct WithdrawMessage {}
