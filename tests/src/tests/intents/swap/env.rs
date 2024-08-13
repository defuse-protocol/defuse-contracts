use impl_tools::autoimpl;
use lazy_static::lazy_static;
use near_sdk::AccountId;
use near_workspaces::{Account, Contract};

use crate::{
    tests::{account::AccountShardExt, intents::swap::SwapIntentShard},
    utils::{ft::FtExt, Sandbox},
};

// HACK: to use with #[rstest] as static variables
lazy_static! {
    pub static ref USER1: AccountId = "user1.test.near".parse().unwrap();
    pub static ref USER2: AccountId = "user2.test.near".parse().unwrap();
    pub static ref FT1: AccountId = "ft1.test.near".parse().unwrap();
    pub static ref FT2: AccountId = "ft2.test.near".parse().unwrap();
    pub static ref ACCOUNT_SHARD1: AccountId = "account-shard1.test.near".parse().unwrap();
    pub static ref ACCOUNT_SHARD2: AccountId = "account-shard2.test.near".parse().unwrap();
}

#[autoimpl(Deref using self.sandbox)]
pub struct Env {
    sandbox: Sandbox,

    pub user1: Account,
    pub user2: Account,
    #[allow(dead_code)]
    pub user3: Account,

    pub swap_intent: Contract,

    pub ft1: Contract,
    pub ft2: Contract,

    pub account_shard1: Contract,
    #[allow(dead_code)]
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

    pub async fn ft_storage_deposit(
        &self,
        token: &AccountId,
        accounts: &[&AccountId],
    ) -> anyhow::Result<()> {
        self.root_account()
            .ft_storage_deposit_many(token, accounts)
            .await
    }

    pub async fn ft_mint(
        &self,
        token: &AccountId,
        account_id: &AccountId,
        amount: u128,
    ) -> anyhow::Result<()> {
        self.root_account().ft_mint(token, account_id, amount).await
    }
}
