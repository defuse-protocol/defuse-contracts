use near_sdk::{AccountId, NearToken};
use near_workspaces::{Account, Contract};
use serde_json::json;

use crate::utils::Sandbox;

pub struct Env {
    pub sandbox: Sandbox,
    pub dao: Account,
    pub mpc_contract: Account,

    pub user: Account,

    pub controller: Contract,
    pub account_shards: Vec<Contract>,
}

impl Env {
    pub async fn create() -> Self {
        let sandbox = Sandbox::new().await.unwrap();
        let dao = sandbox.create_account("dao").await;
        let mpc_contract = sandbox.create_account("mpc").await;
        let user = sandbox.create_account("user").await;

        let controller = sandbox
            .deploy_controller_contract(dao.id(), mpc_contract.id())
            .await;
        Self {
            sandbox,
            dao,
            mpc_contract,
            user,
            controller,
            account_shards: Vec::new(),
        }
    }

    pub async fn deploy_account_shard(&self, name: impl AsRef<str>) -> AccountId {
        let result = self
            .dao
            .call(&self.controller.id(), "deploy_account_shard")
            .args_json(json!({
                "name": name.as_ref(),
            }))
            // TODO: calculate an exact amount needed
            .deposit(NearToken::from_near(5))
            // TODO: calculate an exact amount of Gas needed
            .max_gas()
            .transact()
            .await
            .unwrap();
        assert!(result.is_success(), "deploy_account_shard: {result:#?}");

        result.json().unwrap()
    }
}
