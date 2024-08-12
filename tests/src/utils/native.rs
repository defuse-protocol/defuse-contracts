use near_sdk::{AccountId, NearToken};
use serde_json::json;

pub trait NativeReceiverExt {
    async fn native_on_transfer(
        &self,
        contract: &AccountId,
        amount: NearToken,
        msg: &str,
    ) -> anyhow::Result<bool>;
}

impl NativeReceiverExt for near_workspaces::Account {
    async fn native_on_transfer(
        &self,
        contract: &AccountId,
        amount: NearToken,
        msg: &str,
    ) -> anyhow::Result<bool> {
        self.call(contract, "native_on_transfer")
            .args_json(json!({
                "msg": msg,
            }))
            .deposit(amount)
            .max_gas()
            .transact()
            .await?
            .into_result()?
            .json()
            .map_err(Into::into)
    }
}

impl NativeReceiverExt for near_workspaces::Contract {
    async fn native_on_transfer(
        &self,
        contract: &AccountId,
        amount: NearToken,
        msg: &str,
    ) -> anyhow::Result<bool> {
        self.as_account()
            .native_on_transfer(contract, amount, msg)
            .await
    }
}
