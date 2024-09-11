use defuse_contracts::{defuse::verify::diff::SignedDiffs, nep413::PublicKey};
use near_sdk::AccountId;
use serde_json::json;

pub trait VerifierExt {
    async fn add_public_key(
        &self,
        defuse_contract_id: &AccountId,
        public_key: PublicKey,
    ) -> anyhow::Result<bool>;

    async fn apply_signed_diffs(&self, diffs: SignedDiffs) -> anyhow::Result<()>;
}

impl VerifierExt for near_workspaces::Account {
    async fn add_public_key(
        &self,
        defuse_contract_id: &AccountId,
        public_key: PublicKey,
    ) -> anyhow::Result<bool> {
        // TODO: check bool output
        self.call(defuse_contract_id, "add_public_key")
            .args_json(json!({
                "public_key": public_key,
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?
            .json()
            .map_err(Into::into)
    }

    async fn apply_signed_diffs(&self, diffs: SignedDiffs) -> anyhow::Result<()> {
        self.call(self.id(), "apply_signed_diffs")
            .args_json(json!({
                "diffs": diffs,
            }))
            // TODO: .deposit(NearToken::from_yoctonear(1))?
            .max_gas()
            .transact()
            .await?
            .into_result()
            .map(|_| ())
            .map_err(Into::into)
    }
}

impl VerifierExt for near_workspaces::Contract {
    async fn add_public_key(
        &self,
        defuse_contract_id: &AccountId,
        public_key: PublicKey,
    ) -> anyhow::Result<bool> {
        self.as_account()
            .add_public_key(defuse_contract_id, public_key)
            .await
    }

    async fn apply_signed_diffs(&self, diffs: SignedDiffs) -> anyhow::Result<()> {
        self.as_account().apply_signed_diffs(diffs).await
    }
}
