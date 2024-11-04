pub mod accounts;
// pub mod actions;
mod error;
pub mod events;
pub mod fees;
pub mod intents;
pub mod payload;
pub mod tokens;

use crate::{
    nep245::{receiver::MultiTokenReceiver, MultiTokenCore},
    upgrade::Upgrade,
    utils::access_keys::AccessKeys,
};

pub use self::error::*;

use intents::{relayer::RelayerKeys, IntentsExecutor};
use near_contract_standards::{
    fungible_token::receiver::FungibleTokenReceiver,
    non_fungible_token::core::NonFungibleTokenReceiver,
};
use near_plugins::{AccessControllable, Pausable};
use near_sdk::ext_contract;

use self::{
    accounts::AccountManager,
    tokens::{
        nep141::{FungibleTokenForceWithdrawer, FungibleTokenWithdrawer},
        nep171::{NonFungibleTokenForceWithdrawer, NonFungibleTokenWithdrawer},
        nep245::{MultiTokenForceWithdrawer, MultiTokenWithdrawer},
    },
};

#[ext_contract(ext_defuse)]
pub trait Defuse:
    IntentsExecutor
    + RelayerKeys
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
    + FungibleTokenForceWithdrawer
    + NonFungibleTokenForceWithdrawer
    + MultiTokenForceWithdrawer
    + Pausable
    + Upgrade
    + AccessKeys
{
}
