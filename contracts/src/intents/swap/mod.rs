use near_contract_standards::{
    fungible_token::receiver::FungibleTokenReceiver,
    non_fungible_token::core::NonFungibleTokenReceiver,
};
use near_sdk::ext_contract;

use crate::{mt::core::MultiTokenReceiver, utils::Mutex};

pub use self::{
    action::*, asset::*, cross_chain::*, error::*, intent::*, lost_found::*, native::*, rollback::*,
};

mod action;
mod asset;
mod cross_chain;
mod error;
pub mod events;
mod intent;
mod lost_found;
mod native;
mod rollback;

#[ext_contract(ext_swap_intent)]
pub trait SwapIntentContract:
    NativeReceiver
    + FungibleTokenReceiver
    + NonFungibleTokenReceiver
    + MultiTokenReceiver
    + CrossChainReceiver
    + Rollback
    + LostFound
{
    fn get_intent(&self, id: &IntentId) -> Option<&Mutex<SwapIntent>>;
}
