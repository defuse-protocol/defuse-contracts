mod accounts;
mod diff;
mod tokens;

use accounts::Accounts;
use defuse_contracts::defuse::Defuse;
use near_sdk::{near, BorshStorageKey, PanicOnDefault};

#[derive(PanicOnDefault)]
#[near(contract_state)]
pub struct DefuseImpl {
    accounts: Accounts,
}

#[near]
impl DefuseImpl {
    #[init]
    pub fn new() -> Self {
        Self {
            accounts: Accounts::new(Prefix::Accounts),
        }
    }
}

#[near]
impl Defuse for DefuseImpl {}

#[derive(BorshStorageKey)]
#[near(serializers = [borsh])]
enum Prefix {
    Accounts,
}
