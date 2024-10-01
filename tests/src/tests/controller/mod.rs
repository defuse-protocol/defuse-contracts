use std::sync::LazyLock;

use near_sdk::AccountId;
use near_workspaces::Contract;
use serde_json::json;

use crate::utils::{account::AccountExt, read_wasm, Sandbox};

static CONTROLLER_WASM: LazyLock<Vec<u8>> =
    LazyLock::new(|| read_wasm("defuse_controller_contract"));

pub trait ControllerExt {
    async fn deploy_controller(
        &self,
        controller_id: &str,
        owner: Option<AccountId>,
    ) -> anyhow::Result<Contract>;
}

impl ControllerExt for near_workspaces::Account {
    async fn deploy_controller(
        &self,
        controller_id: &str,
        owner: Option<AccountId>,
    ) -> anyhow::Result<Contract> {
        let contract = self
            .deploy_contract(controller_id, &CONTROLLER_WASM)
            .await?;
        contract
            .call("new")
            .args_json(json!({
                "owner_id": owner.unwrap_or_else(|| self.id().clone()),
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?;

        Ok(contract)
    }
}

#[tokio::test]
async fn test_deploy_contract() {
    let sandbox = Sandbox::new().await.unwrap();
    let controller = sandbox
        .root_account()
        .deploy_controller("controller", None)
        .await
        .unwrap();

    assert_eq!(controller.id().as_str(), "controller.test.near");
}
