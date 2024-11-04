mod core;
mod events;
pub mod receiver;
pub mod resolver;
mod token;

use near_sdk::{json_types::U128, AccountId};

pub use self::{core::*, events::*, token::*};

pub type ClearedApproval = (AccountId, u64, U128);
