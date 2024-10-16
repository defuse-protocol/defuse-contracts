use defuse_contracts::{crypto::SignedPayload, defuse::payload::MultiStandardPayload};
use serde_json::json;

use super::accounts::AccountManagerExt;

mod token_diff;

pub trait ExecuteIntentsExt: AccountManagerExt {
    async fn execute_intents(
        &self,
        intents: impl IntoIterator<Item = SignedPayload<MultiStandardPayload>>,
    ) -> anyhow::Result<()>;
}

impl ExecuteIntentsExt for near_workspaces::Account {
    async fn execute_intents(
        &self,
        intents: impl IntoIterator<Item = SignedPayload<MultiStandardPayload>>,
    ) -> anyhow::Result<()> {
        self.call(self.id(), "execute_intents")
            .args_json(json!({
                "intents": intents.into_iter().collect::<Vec<_>>(),
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()
            .map(|_| ())
            .map_err(Into::into)
    }
}

impl ExecuteIntentsExt for near_workspaces::Contract {
    async fn execute_intents(
        &self,
        intents: impl IntoIterator<Item = SignedPayload<MultiStandardPayload>>,
    ) -> anyhow::Result<()> {
        self.as_account().execute_intents(intents).await
    }
}
