#[cfg(feature = "contract")]
pub mod contract;

pub mod accounts;
pub mod fees;
pub mod intents;
pub mod tokens;

pub use defuse_core as core;
pub use defuse_nep245 as nep245;

use defuse_admin_utils::full_access_keys::FullAccessKeys;
use defuse_controller::ControllerUpgradable;
use defuse_nep245::{receiver::MultiTokenReceiver, MultiTokenCore};
use near_contract_standards::{
    fungible_token::receiver::FungibleTokenReceiver,
    non_fungible_token::core::NonFungibleTokenReceiver,
};
use near_plugins::{AccessControllable, Pausable};
use near_sdk::ext_contract;

use self::{
    accounts::AccountManager,
    intents::{Intents, RelayerKeys},
    tokens::{
        nep141::{FungibleTokenForceWithdrawer, FungibleTokenWithdrawer},
        nep171::{NonFungibleTokenForceWithdrawer, NonFungibleTokenWithdrawer},
        nep245::{MultiTokenForceWithdrawer, MultiTokenWithdrawer},
    },
};

#[ext_contract(ext_defuse)]
pub trait Defuse:
    Intents
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
    + ControllerUpgradable
    + FullAccessKeys
{
}
