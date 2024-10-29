use near_contract_standards::{
    fungible_token::{
        metadata::{FungibleTokenMetadata, FungibleTokenMetadataProvider},
        FungibleTokenCore, FungibleTokenResolver,
    },
    storage_management::StorageManagement,
};
use near_plugins::Ownable;
use near_sdk::{ext_contract, json_types::U128, AccountId};

use crate::utils::access_keys::AccessKeys;

/// Fungible token that allows minting only by its owner.
/// To withdraw, users can call `ft_transfer` on the deployed token,
/// pass token itself as `receiver_id` and provide destination address
/// in `memo` prefixed with `WITHDRAW_TO:`.
#[ext_contract(ext_poa_fungible_token)]
pub trait POAFungibleToken:
    FungibleTokenCore
    + FungibleTokenResolver
    + FungibleTokenMetadataProvider
    + StorageManagement
    + Ownable
    + AccessKeys
{
    /// Sets metadata.
    /// NOTE: MUST attach 1 yⓃ for security purposes.
    fn set_metadata(&mut self, metadata: FungibleTokenMetadata);

    /// Deposits given amount to `owner_id`.
    /// Requires to attach enough Ⓝ to make storage deposit for the user
    /// (see NEP145::storage_balance_bounds()).
    fn ft_deposit(&mut self, owner_id: AccountId, amount: U128, memo: Option<String>);
}

pub const WITHDRAW_MEMO_PREFIX: &str = "WITHDRAW_TO:";

pub fn withdraw_to(address: impl AsRef<str>) -> String {
    format!("{WITHDRAW_MEMO_PREFIX}{}", address.as_ref())
}
