mod accounts;
#[cfg(feature = "beta")]
mod beta;
mod fees;
mod intents;
mod state;
mod tokens;

use accounts::Accounts;
use defuse_contracts::{
    defuse::{Defuse, Result},
    utils::{fees::Pips, UnwrapOrPanic},
};
use impl_tools::autoimpl;
use near_plugins::{access_control, AccessControlRole};
use near_sdk::{near, AccountId, BorshStorageKey, PanicOnDefault};

use self::state::State;

#[derive(AccessControlRole, Clone, Copy)]
enum Role {
    FeesManager,
    #[cfg(feature = "beta")]
    BetaAccess,
}

#[access_control(role_type(Role))]
#[near(contract_state)]
#[autoimpl(Deref using self.state)]
#[autoimpl(DerefMut using self.state)]
#[derive(PanicOnDefault)]
pub struct DefuseImpl {
    accounts: Accounts,
    state: State,
}

#[near]
impl DefuseImpl {
    #[init]
    pub fn new(fee: Pips, fee_collector: AccountId) -> Self {
        // TODO: fee_collector optional, owner by default
        Self {
            accounts: Accounts::new(Prefix::Accounts),
            state: State::new(Prefix::Runtime, fee, fee_collector),
        }
    }
}

impl DefuseImpl {
    #[inline]
    pub fn finalize(&mut self) -> Result<()> {
        self.state.runtime.finalize(&mut self.accounts)
    }
}

impl Drop for DefuseImpl {
    #[inline]
    fn drop(&mut self) {
        self.finalize().unwrap_or_panic()
    }
}

#[near]
impl Defuse for DefuseImpl {}

#[derive(BorshStorageKey)]
#[near(serializers = [borsh])]
enum Prefix {
    Accounts,
    Runtime,
}
