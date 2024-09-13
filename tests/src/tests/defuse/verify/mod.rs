mod diff;

use std::collections::HashMap;

use defuse_contracts::{
    crypto::{PublicKey, Signed},
    defuse::verify::{diff::AccountDiff, payload::MultiStandardPayload},
};
use near_sdk::AccountId;
use serde_json::json;

pub trait VerifierExt {
    async fn add_public_key(
        &self,
        defuse_contract_id: &AccountId,
        public_key: PublicKey,
    ) -> anyhow::Result<()>;

    async fn apply_signed_diffs(
        &self,
        diffs: impl IntoIterator<
            Item = (
                &AccountId,
                impl IntoIterator<Item = Signed<MultiStandardPayload<AccountDiff>>>,
            ),
        >,
    ) -> anyhow::Result<()>;
}

impl VerifierExt for near_workspaces::Account {
    async fn add_public_key(
        &self,
        defuse_contract_id: &AccountId,
        public_key: PublicKey,
    ) -> anyhow::Result<()> {
        // TODO: check bool output
        self.call(defuse_contract_id, "add_public_key")
            .args_json(json!({
                "public_key": public_key,
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?;
        Ok(())
    }

    async fn apply_signed_diffs(
        &self,
        diffs: impl IntoIterator<
            Item = (
                &AccountId,
                impl IntoIterator<Item = Signed<MultiStandardPayload<AccountDiff>>>,
            ),
        >,
    ) -> anyhow::Result<()> {
        self.call(self.id(), "apply_signed_diffs")
            .args_json(json!({
                "diffs": diffs
                    .into_iter()
                    .map(|(account_id, diffs)| (
                        account_id.clone(),
                        diffs.into_iter().collect::<Vec<_>>(),
                    )).collect::<HashMap<_, _>>(),
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
    ) -> anyhow::Result<()> {
        self.as_account()
            .add_public_key(defuse_contract_id, public_key)
            .await
    }

    async fn apply_signed_diffs(
        &self,
        diffs: impl IntoIterator<
            Item = (
                &AccountId,
                impl IntoIterator<Item = Signed<MultiStandardPayload<AccountDiff>>>,
            ),
        >,
    ) -> anyhow::Result<()> {
        self.as_account().apply_signed_diffs(diffs).await
    }
}
