use defuse_contracts::crypto::PublicKey;
use near_sdk::{AccountId, NearToken};
use serde_json::json;

pub trait AccountManagerExt {
    async fn add_public_key(
        &self,
        defuse_contract_id: &AccountId,
        public_key: PublicKey,
    ) -> anyhow::Result<()>;
}

impl AccountManagerExt for near_workspaces::Account {
    async fn add_public_key(
        &self,
        defuse_contract_id: &AccountId,
        public_key: PublicKey,
    ) -> anyhow::Result<()> {
        // TODO: check bool output
        self.call(defuse_contract_id, "add_public_key")
            .deposit(NearToken::from_yoctonear(1))
            .args_json(json!({
                "public_key": public_key,
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?;
        Ok(())
    }
}

impl AccountManagerExt for near_workspaces::Contract {
    async fn add_public_key(
        &self,
        defuse_contract_id: &AccountId,
        public_key: PublicKey,
    ) -> anyhow::Result<()> {
        self.as_account()
            .add_public_key(defuse_contract_id, public_key)
            .await
    }
}
