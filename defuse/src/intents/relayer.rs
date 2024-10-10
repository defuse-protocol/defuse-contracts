use defuse_contracts::{
    defuse::intents::relayer::RelayerKeys,
    utils::{cache::CURRENT_ACCOUNT_ID, UnwrapOrPanic},
};
use near_plugins::{access_control_any, pause, AccessControllable, Pausable};
use near_sdk::{env, near, require, Allowance, Promise, PublicKey};

use crate::{DefuseImpl, DefuseImplExt, Role};

#[near]
impl RelayerKeys for DefuseImpl {
    #[pause(name = "intents")]
    #[payable]
    #[access_control_any(roles(Role::RelayerKeysManager))]
    fn add_relayer_key(&mut self, public_key: PublicKey) -> Promise {
        require!(
            self.relayer_keys.insert(public_key.clone()),
            "key already exists",
        );

        Promise::new(CURRENT_ACCOUNT_ID.clone()).add_access_key_allowance(
            public_key,
            Allowance::limited(env::attached_deposit())
                .ok_or("no deposit attached for allowance")
                .unwrap_or_panic_static_str(),
            CURRENT_ACCOUNT_ID.clone(),
            "execute_signed_intents".into(),
        )
    }

    #[pause(name = "intents")]
    #[access_control_any(roles(Role::RelayerKeysManager))]
    fn delete_relayer_key(&mut self, public_key: PublicKey) -> Promise {
        require!(self.relayer_keys.remove(&public_key), "key not found");

        Promise::new(CURRENT_ACCOUNT_ID.clone()).delete_key(public_key)
    }
}
