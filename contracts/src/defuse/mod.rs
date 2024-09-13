pub mod action;
mod error;
pub mod tokens;
pub mod verify;

pub use self::error::*;

use near_contract_standards::{
    fungible_token::receiver::FungibleTokenReceiver,
    non_fungible_token::core::NonFungibleTokenReceiver,
};
use near_sdk::{ext_contract, json_types::U128, AccountId};
use tokens::nep141::FungibleTokenWithdrawer;

use self::verify::Verifier;

#[ext_contract(ext_defuse)]
pub trait Defuse:
    Verifier + FungibleTokenReceiver + FungibleTokenWithdrawer + NonFungibleTokenReceiver
{
    // TODO: full implementation of NEP-245
    #[allow(clippy::ptr_arg)]
    fn mt_balance_of(&self, account_id: &AccountId, token_id: &String) -> U128;

    #[allow(clippy::ptr_arg)]
    fn mt_batch_balance_of(&self, account_id: &AccountId, token_ids: &Vec<String>) -> Vec<U128>;
}
