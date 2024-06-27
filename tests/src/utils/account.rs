#![allow(dead_code)]
use near_sdk::NearToken;
use near_workspaces::{Account, Contract};

pub trait AccountExt {
    async fn deploy_contract(&self, account_id: &str, wasm: &[u8]) -> anyhow::Result<Contract>;
}

impl AccountExt for Account {
    async fn deploy_contract(&self, account_id: &str, wasm: &[u8]) -> anyhow::Result<Contract> {
        self.create_subaccount(account_id)
            .initial_balance(NearToken::from_near(10))
            .transact()
            .await?
            .into_result()?
            .deploy(wasm.as_ref())
            .await?
            .into_result()
            .map_err(Into::into)
    }
}
