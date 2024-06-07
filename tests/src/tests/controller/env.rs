use near_sdk::AccountId;
use near_workspaces::{Account, Contract};

use crate::utils::{controller::Controller, Sandbox};

pub struct Env {
    pub sandbox: Sandbox,
    pub owner: Account,
    pub mpc_contract: Account,

    pub user: Account,

    pub controller: Contract,
    pub account_shards: Vec<Contract>,
}

impl Env {
    pub async fn create() -> Self {
        let sandbox = Sandbox::new().await.unwrap();
        let owner = sandbox.create_account("owner").await;
        let mpc_contract = sandbox.create_account("mpc").await;

        let user = sandbox.create_account("user").await;

        let controller = sandbox
            .deploy_controller_contract(owner.id(), mpc_contract.id())
            .await;
        Self {
            sandbox,
            owner,
            mpc_contract,
            user,
            controller,
            account_shards: Vec::new(),
        }
    }

    pub async fn create_and_deploy_account_shard(&self, name: impl AsRef<str>) -> AccountId {
        self.controller
            .create_and_deploy_account_shard(name.as_ref())
            .await
            .unwrap()
    }
}
