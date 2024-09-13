mod env;
mod tokens;
pub mod verify;

use defuse_contracts::defuse::tokens::TokenId;
use lazy_static::lazy_static;
use near_sdk::{json_types::U128, AccountId};
use near_workspaces::Contract;
use serde_json::json;
use verify::VerifierExt;

use crate::utils::{account::AccountExt, read_wasm};

lazy_static! {
    static ref DEFUSE_WASM: Vec<u8> = read_wasm("defuse_contract");
}

pub trait DefuseExt: VerifierExt {
    async fn deploy_defuse(&self, id: &str) -> anyhow::Result<Contract>;

    async fn mt_balance_of(
        &self,
        account_id: &AccountId,
        token_id: &TokenId,
    ) -> anyhow::Result<u128>;

    async fn mt_batch_balance_of(
        &self,
        account_id: &AccountId,
        token_ids: impl IntoIterator<Item = &TokenId>,
    ) -> anyhow::Result<Vec<u128>>;
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

    async fn mt_balance_of(
        &self,
        account_id: &AccountId,
        token_id: &TokenId,
    ) -> anyhow::Result<u128> {
        self.view(self.id(), "mt_balance_of")
            .args_json(json!({
                "account_id": account_id,
                "token_id": token_id,
            }))
            .await?
            .json::<U128>()
            .map(|b| b.0)
            .map_err(Into::into)
    }

    async fn mt_batch_balance_of(
        &self,
        account_id: &AccountId,
        token_ids: impl IntoIterator<Item = &TokenId>,
    ) -> anyhow::Result<Vec<u128>> {
        self.view(self.id(), "mt_batch_balance_of")
            .args_json(json!({
                "account_id": account_id,
                "token_ids": token_ids.into_iter().collect::<Vec<_>>(),
            }))
            .await?
            .json::<Vec<U128>>()
            .map(|bs| bs.into_iter().map(|b| b.0).collect())
            .map_err(Into::into)
    }
}

impl DefuseExt for Contract {
    async fn deploy_defuse(&self, id: &str) -> anyhow::Result<Contract> {
        self.as_account().deploy_defuse(id).await
    }

    async fn mt_balance_of(
        &self,
        account_id: &AccountId,
        token_id: &TokenId,
    ) -> anyhow::Result<u128> {
        self.as_account().mt_balance_of(account_id, token_id).await
    }

    async fn mt_batch_balance_of(
        &self,
        account_id: &AccountId,
        token_ids: impl IntoIterator<Item = &TokenId>,
    ) -> anyhow::Result<Vec<u128>> {
        self.as_account()
            .mt_batch_balance_of(account_id, token_ids)
            .await
    }
}
