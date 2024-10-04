mod accounts;
#[cfg(feature = "beta")]
mod beta;
mod intents;
mod tokens;

use accounts::Accounts;
use defuse_contracts::defuse::{fees::Fees, Defuse};
use near_plugins::{access_control, AccessControlRole};
use near_sdk::{near, BorshStorageKey, PanicOnDefault};
use tokens::TokensBalances;

#[derive(AccessControlRole, Clone, Copy)]
enum Role {
    FeesManager,
    #[cfg(feature = "beta")]
    BetaAccess,
}

#[access_control(role_type(Role))]
#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct DefuseImpl {
    accounts: Accounts,

    total_supplies: TokensBalances,

    fees: Fees,
}

#[near]
impl DefuseImpl {
    #[init]
    pub fn new() -> Self {
        Self {
            accounts: Accounts::new(Prefix::Accounts),
            total_supplies: TokensBalances::new(Prefix::TokenSupplies),
            // TODO
            fees: Default::default(),
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
