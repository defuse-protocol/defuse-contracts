use near_contract_standards::{
    fungible_token::receiver::FungibleTokenReceiver,
    non_fungible_token::core::NonFungibleTokenReceiver,
};
use near_sdk::ext_contract;

use crate::utils::Mutex;

pub use self::{action::*, asset::*, error::*, intent::*, lost_found::*, native::*, rollback::*};

mod action;
mod asset;
mod error;
pub mod events;
mod intent;
mod lost_found;
mod native;
mod rollback;

#[ext_contract(ext_swap_intent)]
pub trait SwapIntentContract:
    NativeAction + FungibleTokenReceiver + NonFungibleTokenReceiver + Rollback + LostFound
{
    fn get_swap_intent(&self, id: &IntentId) -> Option<&Mutex<SwapIntentStatus>>;
}
