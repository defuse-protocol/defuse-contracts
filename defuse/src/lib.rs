mod accounts;
#[cfg(feature = "beta")]
mod beta;
mod fees;
mod intents;
mod state;
mod tokens;

use core::iter;
use std::collections::HashMap;

use accounts::Accounts;
use defuse_contracts::{
    defuse::{Defuse, Result},
    utils::{fees::Pips, UnwrapOrPanic},
};
use impl_tools::autoimpl;
use near_plugins::{access_control, AccessControlRole};
use near_sdk::{near, require, AccountId, BorshStorageKey, PanicOnDefault};

use self::state::State;

#[near(serializers = [json])]
#[derive(AccessControlRole, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Role {
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
    pub fn new(
        fee: Pips,
        fee_collector: AccountId,
        super_admins: Vec<AccountId>,
        admins: HashMap<Role, Vec<AccountId>>,
        grantees: HashMap<Role, Vec<AccountId>>,
    ) -> Self {
        let mut contract = Self {
            accounts: Accounts::new(Prefix::Accounts),
            state: State::new(Prefix::Runtime, fee, fee_collector),
        };

        let mut acl = contract.acl_get_or_init();
        require!(
            super_admins
                .into_iter()
                .all(|super_admin| acl.add_super_admin_unchecked(&super_admin))
                && admins
                    .into_iter()
                    .flat_map(|(role, admins)| iter::repeat(role).zip(admins))
                    .all(|(role, admin)| acl.add_admin_unchecked(role, &admin))
                && grantees
                    .into_iter()
                    .flat_map(|(role, grantees)| iter::repeat(role).zip(grantees))
                    .all(|(role, grantee)| acl.grant_role_unchecked(role, &grantee)),
            "failed to set roles"
        );

        contract
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
