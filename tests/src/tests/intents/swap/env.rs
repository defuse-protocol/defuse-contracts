use std::sync::LazyLock;

use defuse_poa_factory_contract::Role as POAFactoryRole;
use impl_tools::autoimpl;
use near_sdk::AccountId;
use near_workspaces::{Account, Contract};

use crate::{
    tests::{
        account::AccountShardExt, intents::swap::SwapIntentShard, poa::factory::PoAFactoryExt,
    },
    utils::{ft::FtExt, Sandbox},
};

// HACK: to use with #[rstest] as static variables
pub static USER1: LazyLock<AccountId> = LazyLock::new(|| "user1.test.near".parse().unwrap());
pub static USER2: LazyLock<AccountId> = LazyLock::new(|| "user2.test.near".parse().unwrap());
pub static FT1: LazyLock<AccountId> =
    LazyLock::new(|| "ft1.poa-factory.test.near".parse().unwrap());
pub static FT2: LazyLock<AccountId> =
    LazyLock::new(|| "ft2.poa-factory.test.near".parse().unwrap());
pub static ACCOUNT_SHARD1: LazyLock<AccountId> =
    LazyLock::new(|| "account-shard1.test.near".parse().unwrap());
pub static ACCOUNT_SHARD2: LazyLock<AccountId> =
    LazyLock::new(|| "account-shard2.test.near".parse().unwrap());

#[autoimpl(Deref using self.sandbox)]
pub struct Env {
    sandbox: Sandbox,

    pub user1: Account,
    pub user2: Account,
    #[allow(dead_code)]
    pub user3: Account,

    pub swap_intent: Contract,

    pub poa_factory: Contract,

    pub ft1: AccountId,
    pub ft2: AccountId,

    pub account_shard1: Contract,
    #[allow(dead_code)]
    pub account_shard2: Contract,
}

impl Env {
    pub async fn new() -> anyhow::Result<Self> {
        let sandbox = Sandbox::new().await?;
        let root = sandbox.root_account();

        let poa_factory = root
            .deploy_poa_factory(
                "poa-factory",
                [root.id().clone()],
                [
                    (POAFactoryRole::TokenDeployer, [root.id().clone()]),
                    (POAFactoryRole::TokenDepositer, [root.id().clone()]),
                ],
                [
                    (POAFactoryRole::TokenDeployer, [root.id().clone()]),
                    (POAFactoryRole::TokenDepositer, [root.id().clone()]),
                ],
            )
            .await?;

        Ok(Self {
            user1: sandbox.create_account("user1").await,
            user2: sandbox.create_account("user2").await,
            user3: sandbox.create_account("user3").await,
            swap_intent: root.deploy_swap_intent_shard("swap-intent").await?,
            ft1: root
                .poa_factory_deploy_token(poa_factory.id(), "ft1", None)
                .await?,
            ft2: root
                .poa_factory_deploy_token(poa_factory.id(), "ft2", None)
                .await?,
            account_shard1: root.deploy_account_shard("account-shard1", None).await?,
            account_shard2: root.deploy_account_shard("account-shard2", None).await?,
            poa_factory,
            sandbox,
        })
    }

    pub async fn ft_storage_deposit(
        &self,
        token: &AccountId,
        accounts: &[&AccountId],
    ) -> anyhow::Result<()> {
        self.ft_storage_deposit_many(token, accounts).await
    }

    pub async fn ft_mint(
        &self,
        token: &AccountId,
        account_id: &AccountId,
        amount: u128,
    ) -> anyhow::Result<()> {
        self.poa_factory_ft_deposit(
            self.poa_factory.id(),
            token
                .as_str()
                .strip_suffix(&format!(".{}", self.poa_factory.id()))
                .expect("can't ming this token"),
            account_id,
            amount,
            None,
            None,
        )
        .await
    }
}
