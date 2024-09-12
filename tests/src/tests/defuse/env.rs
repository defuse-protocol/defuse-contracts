use defuse_contracts::defuse::{action::Action, token::DepositMessage};
use near_sdk::AccountId;
use near_workspaces::{Account, Contract};

use crate::utils::{ft::FtExt, Sandbox};

use super::{tokens::nep141::DefuseFtReceiver, verify::VerifierExt, DefuseExt};

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

        let s = Self {
            user1: sandbox.create_account("user1").await,
            user2: sandbox.create_account("user2").await,
            defuse: root.deploy_defuse("defuse").await?,
            ft1: root.deploy_ft_token("ft1").await?,
            ft2: root.deploy_ft_token("ft2").await?,
            sandbox,
        };

        s.ft_storage_deposit(s.ft1.id(), &[s.user1.id(), s.user2.id(), s.defuse.id()])
            .await?;
        s.ft_storage_deposit(s.ft2.id(), &[s.user1.id(), s.user2.id(), s.defuse.id()])
            .await?;

        // NOTE: near_workspaces uses the same signer all subaccounts
        s.user1
            .add_public_key(
                s.defuse.id(),
                // HACK: near_worspaces does not expose near_crypto API
                s.user1.secret_key().public_key().to_string().parse()?,
            )
            .await?;
        s.user2
            .add_public_key(
                s.defuse.id(),
                s.user2.secret_key().public_key().to_string().parse()?,
            )
            .await?;

        Ok(s)
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

    pub async fn defuse_ft_mint(
        &self,
        token_id: &AccountId,
        amount: u128,
        to: AccountId,
        actions: Vec<Action>,
    ) -> anyhow::Result<()> {
        self.sandbox
            .root_account()
            .defuse_ft_deposit(
                self.defuse.id(),
                token_id,
                amount,
                &DepositMessage {
                    deposit_to: Some(to),
                    actions,
                },
            )
            .await
    }
}
