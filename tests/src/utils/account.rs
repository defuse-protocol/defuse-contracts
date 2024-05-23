#![allow(dead_code)]
use near_workspaces::AccountId;

pub trait Account {
    async fn add_account(&self, account_id: AccountId);
}

impl Account for near_workspaces::Account {
    async fn add_account(&self, _account_id: AccountId) {
        todo!()
    }
}
