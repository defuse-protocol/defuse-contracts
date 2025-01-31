use defuse::core::crypto::PublicKey;
use near_sdk::{AccountId, NearToken};
use serde_json::json;

pub trait AccountManagerExt {
    async fn add_public_key(
        &self,
        defuse_contract_id: &AccountId,
        public_key: PublicKey,
    ) -> anyhow::Result<()>;

    async fn defuse_has_public_key(
        &self,
        defuse_contract_id: &AccountId,
        account_id: &AccountId,
        public_key: &PublicKey,
    ) -> anyhow::Result<bool>;

    async fn has_public_key(
        &self,
        account_id: &AccountId,
        public_key: &PublicKey,
    ) -> anyhow::Result<bool>;
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

    async fn defuse_has_public_key(
        &self,
        defuse_contract_id: &AccountId,
        account_id: &AccountId,
        public_key: &PublicKey,
    ) -> anyhow::Result<bool> {
        self.view(defuse_contract_id, "has_public_key")
            .args_json(json!({
                "account_id": account_id,
                "public_key": public_key,
            }))
            .await?
            .json()
            .map_err(Into::into)
    }

    async fn has_public_key(
        &self,
        account_id: &AccountId,
        public_key: &PublicKey,
    ) -> anyhow::Result<bool> {
        self.defuse_has_public_key(self.id(), account_id, public_key)
            .await
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

    async fn defuse_has_public_key(
        &self,
        defuse_contract_id: &AccountId,
        account_id: &AccountId,
        public_key: &PublicKey,
    ) -> anyhow::Result<bool> {
        self.as_account()
            .defuse_has_public_key(defuse_contract_id, account_id, public_key)
            .await
    }

    async fn has_public_key(
        &self,
        account_id: &AccountId,
        public_key: &PublicKey,
    ) -> anyhow::Result<bool> {
        self.as_account()
            .has_public_key(account_id, public_key)
            .await
    }
}
