use defuse_contracts::{method_name, upgrade::Upgrade, utils::cache::CURRENT_ACCOUNT_ID};
use near_plugins::{access_control_any, AccessControllable};
use near_sdk::{assert_one_yocto, near, Gas, NearToken, Promise};

use crate::{DefuseImpl, DefuseImplExt, Role};

const STATE_MIGRATE_FUNCTION: &str = method_name!(DefuseImpl::state_migrate);
const STATE_MIGRATE_DEFAULT_GAS: Gas = Gas::from_tgas(5);

#[near]
impl Upgrade for DefuseImpl {
    #[access_control_any(roles(Role::DAO))]
    #[payable]
    fn upgrade(
        &mut self,
        #[serializer(borsh)] code: Vec<u8>,
        #[serializer(borsh)] state_migration_gas: Option<Gas>,
    ) -> Promise {
        assert_one_yocto();
        Promise::new(CURRENT_ACCOUNT_ID.clone())
            .deploy_contract(code)
            .function_call(
                STATE_MIGRATE_FUNCTION.into(),
                Vec::new(),
                NearToken::from_yoctonear(0),
                state_migration_gas.unwrap_or(STATE_MIGRATE_DEFAULT_GAS),
            )
    }

    #[private]
    fn state_migrate(&mut self) {}
}
