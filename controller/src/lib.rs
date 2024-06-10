use near_plugins::{access_control, AccessControlRole, AccessControllable, Ownable};
use near_sdk::{
    env, log, near,
    serde_json::{self, json},
    AccountId, Gas, NearToken, PanicOnDefault, Promise, PromiseError,
};

// TODO: use near_workspaces::compile_project("./some/path").await?;
const ACCOUNT_WASM: &[u8] = include_bytes!("../../res/defuse-account-contract.wasm");

#[derive(AccessControlRole, Clone, Copy)]
pub enum Role {
    DAO,
}

#[near(contract_state, serializers=[borsh])]
#[derive(PanicOnDefault, Ownable)]
#[access_control(role_type(Role))]
pub struct Controller {
    mpc_contract_id: AccountId,
    account_shard_code: Vec<u8>,
}

#[near]
impl Controller {
    #[init]
    #[must_use]
    pub fn new(owner_id: AccountId, mpc_contract_id: AccountId) -> Self {
        // TODO
        // assert!(!env::state_exists(), "Already initialized");

        let mut contract = Self {
            mpc_contract_id,
            account_shard_code: Vec::new(),
        };
        contract.owner_set(Some(owner_id));
        contract
    }

    #[payable]
    pub fn create_and_deploy_account_shard(&mut self, name: String) -> Promise {
        let self_id = env::current_account_id();

        let account_shard_id: AccountId = format!("{name}.{self_id}").parse().unwrap();
        // TODO: validate?
        // assert!(env::is_valid_account_id(account_shard_id));

        let attached = env::attached_deposit();
        // TODO: check if attached_deposit is enough to stake for the storage?

        Promise::new(account_shard_id.clone())
            .create_account()
            // .add_full_access_key(public_key)
            .transfer(attached)
            .deploy_contract(ACCOUNT_WASM.to_vec())
            .function_call(
                "new".to_string(),
                serde_json::to_vec(&json!({
                    "owner_id": &self_id,
                    "mpc_contract_id": &self.mpc_contract_id,
                }))
                .unwrap(),
                NearToken::from_yoctonear(0),
                Gas::from_tgas(3), // TODO
            )
            .then(Self::ext(self_id).create_and_deploy_account_shard_callback(
                account_shard_id.clone(),
                env::predecessor_account_id(),
                attached,
            ))
    }

    #[private]
    pub fn create_and_deploy_account_shard_callback(
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

    pub fn release_version() {}

    pub fn upgrade_all(&mut self) {}

    pub fn pause_all(&self) {}
}
