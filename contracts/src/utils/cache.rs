use lazy_static::lazy_static;
use near_sdk::{env, AccountId};

lazy_static! {
    /// Cached [`env::current_account_id()`]
    pub static ref CURRENT_ACCOUNT_ID: AccountId = env::current_account_id();
    /// Cached [`env::predecessor_account_id()`]
    pub static ref PREDECESSOR_ACCOUNT_ID: AccountId = env::predecessor_account_id();
}
