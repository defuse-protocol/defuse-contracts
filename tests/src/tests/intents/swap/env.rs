use impl_tools::autoimpl;
use near_workspaces::{Account, Contract};

use crate::{
    tests::{account::AccountShardExt, intents::swap::SwapIntentShard},
    utils::{ft::FtExt, Sandbox},
};

#[autoimpl(Deref using self.sandbox)]
pub struct Env {
    sandbox: Sandbox,

    pub user1: Account,
    pub user2: Account,
    pub user3: Account,

    pub swap_intent: Contract,

    pub ft1: Contract,
    pub ft2: Contract,

    pub account_shard1: Contract,
    pub account_shard2: Contract,
}

impl Env {
    pub async fn new() -> anyhow::Result<Self> {
        let sandbox = Sandbox::new().await?;
        let root = sandbox.root_account();

        Ok(Self {
            user1: sandbox.create_account("user1").await,
            user2: sandbox.create_account("user2").await,
            user3: sandbox.create_account("user3").await,
            swap_intent: root.deploy_swap_intent_shard("swap-intent").await?,
            ft1: root.deploy_ft_token("ft1").await?,
            ft2: root.deploy_ft_token("ft2").await?,
            account_shard1: root.deploy_account_shard("account-shard1", None).await?,
            account_shard2: root.deploy_account_shard("account-shard2", None).await?,
            sandbox,
        })
    }
}
