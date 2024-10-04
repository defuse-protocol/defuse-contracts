pub mod accounts;
// pub mod actions;
mod error;
pub mod fees;
pub mod intents;
pub mod message;
pub mod payload;
pub mod tokens;

use crate::nep245::{receiver::MultiTokenReceiver, MultiTokenCore};

pub use self::error::*;
use self::{accounts::AccountManager, tokens::nep141::FungibleTokenWithdrawer};

use intents::SignedIntentExecutor;
use near_contract_standards::{
    fungible_token::receiver::FungibleTokenReceiver,
    non_fungible_token::core::NonFungibleTokenReceiver,
};
use near_plugins::AccessControllable;
use near_sdk::ext_contract;
use tokens::{nep171::NonFungibleTokenWithdrawer, nep245::MultiTokenWithdrawer};

#[ext_contract(ext_defuse)]
pub trait Defuse:
    SignedIntentExecutor
    + AccountManager
    + MultiTokenCore
    // NEP-141 deposits/withdrawals
    + FungibleTokenReceiver
    + FungibleTokenWithdrawer
    // NEP-171 deposits/withdrawals
    + NonFungibleTokenReceiver
    + NonFungibleTokenWithdrawer
    // NEP-245 deposits/withdrawals
    + MultiTokenReceiver
    + MultiTokenWithdrawer
    // Governance
    + AccessControllable
{
}
