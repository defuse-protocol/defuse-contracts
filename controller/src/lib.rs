use near_plugins::{access_control, access_control_any, AccessControlRole, AccessControllable};
use near_sdk::{
    env, log, near,
    serde_json::{self, json},
    store::LookupSet,
    AccountId, BorshStorageKey, Gas, NearToken, PanicOnDefault, Promise, PromiseError,
};

// TODO: use near_workspaces::compile_project("./some/path").await?;
const ACCOUNT_WASM: &[u8] = include_bytes!("../../res/defuse-account-contract.wasm");

#[derive(AccessControlRole, Clone, Copy)]
pub enum Role {
    DAO,
}

#[derive(BorshStorageKey)]
#[near(serializers=[borsh])]
enum Prefix {
    AccountShards,
}

#[near(contract_state, serializers=[borsh])]
#[derive(PanicOnDefault)]
#[access_control(role_type(Role))]
pub struct Controller {
    mpc_contract_id: AccountId,
    account_shards: LookupSet<String>,
}

#[near]
impl Controller {
    #[init]
    #[must_use]
    pub fn new(dao: AccountId, mpc_contract_id: AccountId) -> Self {
        let mut contract = Self {
            mpc_contract_id,
            account_shards: LookupSet::new(Prefix::AccountShards),
        };
        assert!(
            contract.acl_init_super_admin(env::current_account_id()),
            "failed to init super admin",
        );

        assert_eq!(
            contract.acl_grant_role(Role::DAO.into(), dao),
            Some(true),
            "failed to grant DAO role",
        );
        contract
    }

    #[payable]
    #[access_control_any(roles(Role::DAO))]
    pub fn deploy_account_shard(&mut self, name: String) -> Promise {
        let self_id = env::current_account_id();
        let account_shard_id: AccountId = format!("{name}.{self_id}").parse().unwrap();

        // TODO: validate?
        // assert!(env::is_valid_account_id(account_shard_id));

        let attached = env::attached_deposit();
        // TODO: check if attached_deposit is enough to stake for the storage?

        let p = Promise::new(account_shard_id.clone());

        // check if account shard is deployed for the first time
        if self.account_shards.insert(name) {
            p
                // TODO: just in case we lose control over Controller
                // .add_full_access_key(public_key)
                .create_account()
                .transfer(attached)
                // TODO: get code from parameters or storage
                .deploy_contract(ACCOUNT_WASM.to_vec())
                .function_call(
                    "new".to_string(),
                    serde_json::to_vec(&json!({
                        "owner_id": &self_id,
                        "mpc_contract_id": &self.mpc_contract_id,
                    }))
                    .unwrap(),
                    NearToken::from_yoctonear(0),
                    // TODO: calculate exact gas needed and check that we have
                    // enough left
                    Gas::from_tgas(3),
                )
        } else {
            // upgrade otherwise
            p.transfer(attached)
                // TODO: use AccountContract::ext()
                .function_call(
                    "upgrade".to_string(),
                    serde_json::to_vec(&json!({
                        // TODO: get code from parameters or storage
                        "code": ACCOUNT_WASM,
                    }))
                    .unwrap(),
                    NearToken::from_yoctonear(0),
                    // TODO: calculate exact gas needed and check that we have
                    // enough left
                    Gas::from_tgas(150),
                )
        }
        .then(
            // refund the user back in case deploy failed
            Self::ext(self_id).deploy_account_shard_callback(
                account_shard_id.clone(),
                env::predecessor_account_id(),
                attached,
            ),
        )
    }

    #[private]
    pub fn deploy_account_shard_callback(
        &self,
        account_shard_id: AccountId,
        user: AccountId,
        attached: NearToken,
        #[callback_result] create_deploy_result: Result<(), PromiseError>,
    ) -> AccountId {
        if create_deploy_result.is_err() {
            log!("Failed to create {account_shard_id}, returning {attached}yⓃ to {user}",);
            Promise::new(user).transfer(attached);
            env::abort()
        };

        log!("Successfully deployed {account_shard_id}",);
        account_shard_id
    }
}
