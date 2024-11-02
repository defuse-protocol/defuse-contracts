mod accounts;
mod admin;
mod fees;
mod intents;
mod state;
mod tokens;

use core::iter;
use std::collections::HashMap;

use defuse_contracts::{
    defuse::{Defuse, Result},
    utils::{fees::Pips, UnwrapOrPanic},
};
use impl_tools::autoimpl;
use near_plugins::{access_control, AccessControlRole, AccessControllable, Pausable, Upgradable};
use near_sdk::{
    borsh::BorshDeserialize, near, require, store::LookupSet, AccountId, BorshStorageKey,
    PanicOnDefault,
};

use self::{accounts::Accounts, state::State};

#[near(serializers = [json])]
#[derive(AccessControlRole, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Role {
    PauseManager,

    UpgradeCodeStager,
    UpgradeCodeDeployer,
    UpgradeDurationManager,

    FeesManager,
    RelayerKeysManager,
}

#[access_control(role_type(Role))]
#[derive(Pausable, Upgradable, PanicOnDefault)]
#[pausable(manager_roles(Role::PauseManager))]
#[upgradable(access_control_roles(
    code_stagers(Role::UpgradeCodeStager),
    code_deployers(Role::UpgradeCodeDeployer),
    duration_initializers(Role::UpgradeDurationManager),
    duration_update_stagers(Role::UpgradeDurationManager),
    duration_update_appliers(Role::UpgradeDurationManager),
))]
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
pub struct DefuseImpl {
    accounts: Accounts,
    state: State,

    relayer_keys: LookupSet<near_sdk::PublicKey>,
}

#[near]
impl DefuseImpl {
    #[init]
    pub fn new(
        wnear_id: AccountId,
        fee: Pips,
        fee_collector: AccountId,
        super_admins: Vec<AccountId>,
        admins: HashMap<Role, Vec<AccountId>>,
        grantees: HashMap<Role, Vec<AccountId>>,
        staging_duration: Option<near_sdk::Duration>,
    ) -> Self {
        let mut contract = Self {
            accounts: Accounts::new(Prefix::Accounts),
            state: State::new(Prefix::State, wnear_id, fee, fee_collector),
            relayer_keys: LookupSet::new(Prefix::RelayerKeys),
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

        if let Some(staging_duration) = staging_duration {
            contract.up_set_staging_duration_unchecked(staging_duration);
        }

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
    State,
    RelayerKeys,
}
