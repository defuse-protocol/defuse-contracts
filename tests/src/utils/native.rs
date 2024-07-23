use defuse_contracts::intents::swap::SwapIntentAction;
use near_sdk::{AccountId, NearToken};
use serde_json::json;

pub trait NativeActionExt {
    async fn native_action(
        &self,
        contract: &AccountId,
        amount: NearToken,
        action: SwapIntentAction,
    ) -> anyhow::Result<bool>;
}

impl NativeActionExt for near_workspaces::Account {
    async fn native_action(
        &self,
        contract: &AccountId,
        amount: NearToken,
        action: SwapIntentAction,
    ) -> anyhow::Result<bool> {
        self.call(contract, "native_action")
            .args_json(json!({
                "action": action,
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

impl NativeActionExt for near_workspaces::Contract {
    async fn native_action(
        &self,
        contract: &AccountId,
        amount: NearToken,
        action: SwapIntentAction,
    ) -> anyhow::Result<bool> {
        self.as_account()
            .native_action(contract, amount, action)
            .await
    }
}
