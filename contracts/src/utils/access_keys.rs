use near_sdk::{ext_contract, Promise, PublicKey};

#[ext_contract(ext_super_admin)]
pub trait AccessKeys {
    fn add_full_access_key(&mut self, public_key: PublicKey) -> Promise;
    fn delete_key(&mut self, public_key: PublicKey) -> Promise;
}
