mod accounts;
mod diff;
mod tokens;

use accounts::Accounts;
use defuse_contracts::defuse::Defuse;
use near_sdk::{near, BorshStorageKey, PanicOnDefault};
use tokens::TokensBalances;

#[derive(PanicOnDefault)]
#[near(contract_state)]
pub struct DefuseImpl {
    accounts: Accounts,
    total_supplies: TokensBalances,
}

#[near]
impl DefuseImpl {
    #[init]
    pub fn new() -> Self {
        Self {
            accounts: Accounts::new(Prefix::Accounts),
            total_supplies: TokensBalances::new(Prefix::TokenSupplies),
        }
    }
}

#[near]
impl Defuse for DefuseImpl {}

#[derive(BorshStorageKey)]
#[near(serializers = [borsh])]
enum Prefix {
    Accounts,
    TokenSupplies,
}
