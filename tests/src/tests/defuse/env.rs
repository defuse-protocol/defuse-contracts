use near_sdk::AccountId;
use near_workspaces::{Account, Contract};

use crate::utils::{ft::FtExt, Sandbox};

use super::{verify::VerifierExt, DefuseExt};

pub struct Env {
    sandbox: Sandbox,

    pub user1: Account,
    pub user2: Account,

    pub defuse: Contract,

    pub ft1: Contract,
    pub ft2: Contract,
}

impl Env {
    pub async fn new() -> anyhow::Result<Self> {
        let sandbox = Sandbox::new().await?;
        let root = sandbox.root_account();

        let user1 = sandbox.create_account("user1").await;
        let user2 = sandbox.create_account("user2").await;

        let defuse = root.deploy_defuse("defuse").await?;

        // NOTE: near_workspaces uses the same signer all subaccounts
        user1
            .add_public_key(
                defuse.id(),
                // HACK: near_worspaces does not expose near_crypto API
                user1.secret_key().public_key().to_string().parse()?,
            )
            .await?;
        user2
            .add_public_key(
                defuse.id(),
                user2.secret_key().public_key().to_string().parse()?,
            )
            .await?;

        Ok(Self {
            user1,
            user2,
            defuse,
            ft1: root.deploy_ft_token("ft1").await?,
            ft2: root.deploy_ft_token("ft2").await?,
            sandbox,
        })
    }

    pub async fn ft_storage_deposit(
        &self,
        token: &AccountId,
        accounts: &[&AccountId],
    ) -> anyhow::Result<()> {
        self.sandbox
            .root_account()
            .ft_storage_deposit_many(token, accounts)
            .await
    }

    pub async fn ft_mint(
        &self,
        token: &AccountId,
        account_id: &AccountId,
        amount: u128,
    ) -> anyhow::Result<()> {
        self.sandbox
            .root_account()
            .ft_mint(token, account_id, amount)
            .await
    }
}
