use lazy_static::lazy_static;
use near_sdk::AccountId;
use near_workspaces::Contract;
use serde_json::json;

use crate::utils::{account::AccountExt, read_wasm, Sandbox};

lazy_static! {
    static ref CONTROLLER_WASM: Vec<u8> = read_wasm("defuse-controller-contract");
}

pub trait ControllerExt {
    async fn deploy_controller(
        &self,
        controller_id: impl AsRef<str>,
        owner: impl Into<Option<AccountId>>,
    ) -> Contract;
}

impl ControllerExt for near_workspaces::Account {
    async fn deploy_controller(
        &self,
        controller_id: impl AsRef<str>,
        owner: impl Into<Option<AccountId>>,
    ) -> Contract {
        let contract = self.deploy_contract(controller_id, &CONTROLLER_WASM).await;
        contract
            .call("new")
            .args_json(json!({
                "owner_id": owner.into().unwrap_or(self.id().clone()),
            }))
            .max_gas()
            .transact()
            .await
            .unwrap()
            .into_result()
            .unwrap();

        contract
    }
}

#[tokio::test]
async fn test_deploy_contract() {
    let sandbox = Sandbox::new().await.unwrap();
    let controller = sandbox
        .root_account()
        .deploy_controller("controller", None)
        .await;

    assert_eq!(controller.id().as_str(), "controller.test.near");
}
