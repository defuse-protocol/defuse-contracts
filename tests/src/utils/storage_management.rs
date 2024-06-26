use near_contract_standards::storage_management::StorageBalance;
use near_sdk::{AccountId, NearToken};
use serde_json::json;

pub trait StorageManagementExt {
    async fn storage_deposit(
        &self,
        contract_id: &AccountId,
        account_id: impl Into<Option<AccountId>>,
        deposit: NearToken,
    ) -> anyhow::Result<StorageBalance>;
    async fn storage_unregister(
        &self,
        contract_id: &AccountId,
        force: impl Into<Option<bool>>,
    ) -> anyhow::Result<bool>;
}

impl StorageManagementExt for near_workspaces::Account {
    async fn storage_deposit(
        &self,
        contract_id: &AccountId,
        account_id: impl Into<Option<AccountId>>,
        deposit: NearToken,
    ) -> anyhow::Result<StorageBalance> {
        self.call(contract_id, "storage_deposit")
            .args_json(json!({
                "account_id": account_id.into().unwrap_or(self.id().clone())
            }))
            .deposit(deposit)
            .max_gas()
            .transact()
            .await?
            .into_result()?
            .json()
            .map_err(Into::into)
    }

    async fn storage_unregister(
        &self,
        contract_id: &AccountId,
        force: impl Into<Option<bool>>,
    ) -> anyhow::Result<bool> {
        self.call(contract_id, "storage_unregister")
            .args_json(json!({
                "force": force.into(),
            }))
            .deposit(NearToken::from_yoctonear(1))
            .max_gas()
            .transact()
            .await?
            .into_result()?
            .json()
            .map_err(Into::into)
    }
}
