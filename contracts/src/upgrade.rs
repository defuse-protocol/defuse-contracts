use near_sdk::{ext_contract, Gas, Promise};

#[ext_contract(ext_upgrade)]
pub trait Upgrade {
    fn upgrade(
        &mut self,
        #[serializer(borsh)] code: Vec<u8>,
        #[serializer(borsh)] state_migration_gas: Option<Gas>,
    ) -> Promise;

    fn state_migrate(&mut self);
}
