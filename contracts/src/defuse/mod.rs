pub mod accounts;
pub mod diff;
mod error;
pub mod payload;
pub mod tokens;

use crate::nep245::{receiver::MultiTokenReceiver, MultiTokenCore};

pub use self::error::*;
use self::{accounts::AccountManager, tokens::nep141::FungibleTokenWithdrawer};

use diff::SignedDiffer;
use near_contract_standards::{
    fungible_token::receiver::FungibleTokenReceiver,
    non_fungible_token::core::NonFungibleTokenReceiver,
};
use near_sdk::ext_contract;

#[ext_contract(ext_defuse)]
pub trait Defuse:
    SignedDiffer
    + AccountManager
    + MultiTokenCore
    + FungibleTokenReceiver
    + NonFungibleTokenReceiver
    + MultiTokenReceiver
// TODO: implement withdrawals as part of MultiTokenCore, not FungibleTokenWithdrawer
{
}
