mod accounts;
mod admin;
pub mod config;
mod fees;
mod intents;
mod state;
mod tokens;
mod upgrade;

use core::iter;

use defuse_core::Result;

use impl_tools::autoimpl;
use near_plugins::{access_control, AccessControlRole, AccessControllable, Pausable};
use near_sdk::{
    borsh::BorshDeserialize, near, require, store::LookupSet, BorshStorageKey, PanicOnDefault,
};

use crate::Defuse;

use self::{
    accounts::Accounts,
    config::{DefuseConfig, RolesConfig},
    state::ContractState,
};

#[near(serializers = [json])]
#[derive(AccessControlRole, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Role {
    DAO,

    FeesManager,
    RelayerKeysManager,

    UnrestrictedWithdrawer,

    PauseManager,
    Upgrader,
}

#[access_control(role_type(Role))]
#[derive(Pausable, PanicOnDefault)]
#[pausable(manager_roles(Role::DAO, Role::PauseManager))]
#[near(contract_state, contract_metadata(
    // TODO: remove when this PR is merged:
    // https://github.com/near/near-sdk-rs/pull/1249
    version = "unknown",
    link = "https://github.com/defuse-protocol/defuse-contracts",
    standard(standard = "dip4", version = "0.1.0"),
    standard(standard = "nep245", version = "1.0.0"),
))]
#[autoimpl(Deref using self.state)]
#[autoimpl(DerefMut using self.state)]
pub struct Contract {
    accounts: Accounts,
    state: ContractState,

    relayer_keys: LookupSet<near_sdk::PublicKey>,
}

#[near]
impl Contract {
    #[init]
    pub fn new(config: DefuseConfig) -> Self {
        let mut contract = Self {
            accounts: Accounts::new(Prefix::Accounts),
            state: ContractState::new(Prefix::State, config.wnear_id, config.fees),
            relayer_keys: LookupSet::new(Prefix::RelayerKeys),
        };
        contract.init_acl(config.roles);
        contract
    }

    fn init_acl(&mut self, roles: RolesConfig) {
        let mut acl = self.acl_get_or_init();
        require!(
            roles
                .super_admins
                .into_iter()
                .all(|super_admin| acl.add_super_admin_unchecked(&super_admin))
                && roles
                    .admins
                    .into_iter()
                    .flat_map(|(role, admins)| iter::repeat(role).zip(admins))
                    .all(|(role, admin)| acl.add_admin_unchecked(role, &admin))
                && roles
                    .grantees
                    .into_iter()
                    .flat_map(|(role, grantees)| iter::repeat(role).zip(grantees))
                    .all(|(role, grantee)| acl.grant_role_unchecked(role, &grantee)),
            "failed to set roles"
        );
    }
}

#[near]
impl Defuse for Contract {}

// TODO:?
// impl Drop for Contract {
//     fn drop(&mut self) {
//         mem::take(&mut self.transfer_runtime)
//             .finalize()
//             .unwrap_or_panic()
//             .emit();
//     }
// }

#[derive(BorshStorageKey)]
#[near(serializers = [borsh])]
enum Prefix {
    Accounts,
    State,
    RelayerKeys,
}
