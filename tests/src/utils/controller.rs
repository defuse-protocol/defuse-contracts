use near_sdk::{AccountId, NearToken};
use near_workspaces::Contract;
use serde_json::json;

// TODO: move trait to separate crate `defuse-controller-contract-api`,
// implement in another crate `defuse-controller-contract` and import here
// the trait only from `defuse-controller-contract-api`.
// Not sure whether it's a good solution as #[near_bindgen] doesn't allow params like &str and requires an owned version
pub trait Controller {
    async fn create_and_deploy_account_shard(&self, account_id: &str) -> anyhow::Result<AccountId>;
}

impl Controller for Contract {
    async fn create_and_deploy_account_shard(&self, name: &str) -> anyhow::Result<AccountId> {
        let result = self
            .call("create_and_deploy_account_shard")
            .args_json(json!({
                "name": name,
            }))
            // TODO: calculate an exact amount needed
            .deposit(NearToken::from_near(5))
            // TODO: calculate an exact amount of Gas needed
            .max_gas()
            .transact()
            .await
            .unwrap();
        assert!(
            result.is_success(),
            "create_and_deploy_account_shard: {result:#?}"
        );

        result.json().map_err(Into::into)
    }
}
