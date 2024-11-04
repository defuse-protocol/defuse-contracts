use defuse_contracts::utils::{access_keys::AccessKeys, cache::CURRENT_ACCOUNT_ID};
use near_plugins::{access_control_any, AccessControllable};
use near_sdk::{assert_one_yocto, near, Promise, PublicKey};

use crate::{DefuseImpl, DefuseImplExt, Role};

#[near]
impl AccessKeys for DefuseImpl {
    #[access_control_any(roles(Role::DAO))]
    #[payable]
    fn add_full_access_key(&mut self, public_key: PublicKey) -> Promise {
        assert_one_yocto();
        Promise::new(CURRENT_ACCOUNT_ID.clone()).add_full_access_key(public_key)
    }

    #[access_control_any(roles(Role::DAO))]
    #[payable]
    fn delete_key(&mut self, public_key: PublicKey) -> Promise {
        assert_one_yocto();
        Promise::new(CURRENT_ACCOUNT_ID.clone()).delete_key(public_key)
    }
}
