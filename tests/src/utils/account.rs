#![allow(dead_code)]
use near_sdk::NearToken;
use near_workspaces::{Account, AccountId, Contract};

// pub trait Account {
//     async fn add_account(&self, account_id: AccountId);
// }

// impl Account for near_workspaces::Account {
//     async fn add_account(&self, _account_id: AccountId) {
//         todo!()
//     }
// }

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
