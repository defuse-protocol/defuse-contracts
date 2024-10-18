use defuse_contracts::utils::{
    access_keys::AccessKeys,
    cache::{CURRENT_ACCOUNT_ID, PREDECESSOR_ACCOUNT_ID},
};
use near_sdk::{near, require, Promise, PublicKey};

use crate::{DefuseImpl, DefuseImplExt};

#[near]
impl AccessKeys for DefuseImpl {
    fn add_full_access_key(&mut self, public_key: PublicKey) -> Promise {
        require!(
            self.acl_get_or_init()
                .is_super_admin(&PREDECESSOR_ACCOUNT_ID),
            "super admin required",
        );

        Promise::new(CURRENT_ACCOUNT_ID.clone()).add_full_access_key(public_key)
    }

    fn delete_key(&mut self, public_key: PublicKey) -> Promise {
        require!(
            self.acl_get_or_init()
                .is_super_admin(&PREDECESSOR_ACCOUNT_ID),
            "super admin required",
        );

        Promise::new(CURRENT_ACCOUNT_ID.clone()).delete_key(public_key)
    }
}
