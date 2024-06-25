#![allow(dead_code)]
use near_sdk::NearToken;
use near_workspaces::{Account, Contract};

pub trait AccountExt {
    async fn deploy_contract(&self, account_id: impl AsRef<str>, wasm: &[u8]) -> Contract;
}

impl AccountExt for Account {
    async fn deploy_contract(&self, account_id: impl AsRef<str>, wasm: &[u8]) -> Contract {
        self.create_subaccount(account_id.as_ref())
            .initial_balance(NearToken::from_near(10))
            .transact()
            .await
            .unwrap()
            .into_result()
            .unwrap()
            .deploy(wasm.as_ref())
            .await
            .unwrap()
            .into_result()
            .unwrap()
    }
}
