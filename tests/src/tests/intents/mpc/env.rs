use near_workspaces::{Account, Contract};

use crate::utils::Sandbox;

pub struct Env {
    pub sandbox: Sandbox,

    pub user1: Account,
    pub user2: Account,

    // pub account_shard: Contract,
}

impl Env {
    pub async fn create() -> Self {
        let sandbox = Sandbox::new().await.unwrap();
        let user1 = sandbox.create_account("user1").await;
        let user2 = sandbox.create_account("user2").await;
        Self {
            sandbox,
            user1,
            user2,
        }
    }

    // pub async fn create_intent()
}
