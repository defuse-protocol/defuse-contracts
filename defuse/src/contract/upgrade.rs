use defuse_controller::ControllerUpgradable;
use defuse_near_utils::{method_name, CURRENT_ACCOUNT_ID};
use near_plugins::{access_control_any, AccessControllable};
use near_sdk::{assert_one_yocto, near, Gas, NearToken, Promise};

use super::{Contract, ContractExt, Role};

const STATE_MIGRATE_FUNCTION: &str = method_name!(Contract::state_migrate);
const STATE_MIGRATE_DEFAULT_GAS: Gas = Gas::from_tgas(5);

#[near]
impl ControllerUpgradable for Contract {
    #[access_control_any(roles(Role::DAO, Role::Upgrader))]
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
