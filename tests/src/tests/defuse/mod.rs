pub mod accounts;
pub mod diff;
mod env;
mod tokens;

use std::sync::LazyLock;

use accounts::AccountManagerExt;
use defuse_contracts::defuse::tokens::TokenId;
use near_sdk::AccountId;
use near_workspaces::Contract;

use crate::utils::{account::AccountExt, mt::MtExt, read_wasm};

static DEFUSE_WASM: LazyLock<Vec<u8>> = LazyLock::new(|| read_wasm("defuse_contract"));

pub trait DefuseExt: AccountManagerExt + MtExt {
    async fn deploy_defuse(&self, id: &str) -> anyhow::Result<Contract>;

    async fn mt_balance_of(
        &self,
        account_id: &AccountId,
        token_id: &TokenId,
    ) -> anyhow::Result<u128> {
        MtExt::mt_balance_of(self, account_id, &token_id.to_string()).await
    }

    async fn mt_batch_balance_of(
        &self,
        account_id: &AccountId,
        token_ids: impl IntoIterator<Item = &TokenId>,
    ) -> anyhow::Result<Vec<u128>> {
        MtExt::mt_batch_balance_of(
            self,
            account_id,
            token_ids.into_iter().map(ToString::to_string),
        )
        .await
    }
}

impl DefuseExt for near_workspaces::Account {
    async fn deploy_defuse(&self, id: &str) -> anyhow::Result<Contract> {
        let contract = self.deploy_contract(id, &DEFUSE_WASM).await?;

        contract
            .call("new")
            .max_gas()
            .transact()
            .await?
            .into_result()?;

        Ok(contract)
    }
}

impl DefuseExt for Contract {
    async fn deploy_defuse(&self, id: &str) -> anyhow::Result<Contract> {
        self.as_account().deploy_defuse(id).await
    }
}
