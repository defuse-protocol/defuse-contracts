use near_contract_standards::{
    fungible_token::{FungibleTokenCore, FungibleTokenResolver},
    storage_management::StorageManagement,
};
use near_sdk::{ext_contract, json_types::U128, Gas, Promise};

pub const NEAR_WITHDRAW_GAS: Gas = Gas::from_tgas(5);

#[ext_contract(ext_wnear)]
pub trait WNear: FungibleTokenCore + FungibleTokenResolver + StorageManagement {
    fn near_deposit(&mut self);
    fn near_withdraw(&mut self, amount: U128) -> Promise;
}
