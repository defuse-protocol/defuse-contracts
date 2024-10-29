use std::{collections::HashMap, ops::Deref};

use anyhow::anyhow;
use defuse_contract::Role;
use defuse_contracts::{defuse::tokens::DepositMessage, utils::fees::Pips};
use defuse_poa_factory_contract::Role as POAFactoryRole;
use near_sdk::{AccountId, Duration};
use near_workspaces::{Account, Contract};

use crate::{
    tests::poa::factory::PoAFactoryExt,
    utils::{ft::FtExt, Sandbox},
};

use super::{accounts::AccountManagerExt, tokens::nep141::DefuseFtReceiver, DefuseExt};

pub struct Env {
    sandbox: Sandbox,

    pub user1: Account,
    pub user2: Account,
    pub user3: Account,

    pub defuse: Contract,

    pub poa_factory: Contract,

    pub ft1: AccountId,
    pub ft2: AccountId,
    pub ft3: AccountId,
}

impl Env {
    pub fn builder() -> EnvBuilder {
        EnvBuilder::default()
    }

    pub async fn new() -> anyhow::Result<Self> {
        Self::builder().build().await
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

    pub async fn defuse_ft_mint(
        &self,
        token_id: &AccountId,
        amount: u128,
        to: &AccountId,
    ) -> anyhow::Result<()> {
        if self
            .defuse_ft_deposit(
                self.defuse.id(),
                token_id,
                amount,
                DepositMessage::new(to.clone()),
            )
            .await?
            != amount
        {
            return Err(anyhow!("refunded"));
        }
        Ok(())
    }

    pub fn poa_ft1_name(&self) -> &str {
        self.ft1
            .as_str()
            .strip_suffix(&format!(".{}", self.poa_factory.id()))
            .unwrap()
    }

    pub fn poa_ft2_name(&self) -> &str {
        self.ft2
            .as_str()
            .strip_suffix(&format!(".{}", self.poa_factory.id()))
            .unwrap()
    }

    pub fn poa_ft3_name(&self) -> &str {
        self.ft3
            .as_str()
            .strip_suffix(&format!(".{}", self.poa_factory.id()))
            .unwrap()
    }
}

impl Deref for Env {
    type Target = Account;

    fn deref(&self) -> &Self::Target {
        self.sandbox.root_account()
    }
}

#[derive(Debug, Default)]
pub struct EnvBuilder {
    fee: Pips,
    fee_collector: Option<AccountId>,
    super_admins: Vec<AccountId>,
    self_as_super_admin: bool,
    deployer_as_super_admin: bool,
    admins: HashMap<Role, Vec<AccountId>>,
    grantees: HashMap<Role, Vec<AccountId>>,
    staging_duration: Option<Duration>,
}

impl EnvBuilder {
    pub fn fee(mut self, fee: Pips) -> Self {
        self.fee = fee;
        self
    }

    pub fn fee_collector(mut self, fee_collector: AccountId) -> Self {
        self.fee_collector = Some(fee_collector);
        self
    }

    pub fn super_admin(mut self, super_admin: AccountId) -> Self {
        self.super_admins.push(super_admin);
        self
    }

    pub fn self_as_super_admin(mut self) -> Self {
        self.self_as_super_admin = true;
        self
    }

    pub fn deployer_as_super_admin(mut self) -> Self {
        self.deployer_as_super_admin = true;
        self
    }

    pub fn admin(mut self, role: Role, admin: AccountId) -> Self {
        self.admins.entry(role).or_default().push(admin);
        self
    }

    pub fn grantee(mut self, role: Role, grantee: AccountId) -> Self {
        self.grantees.entry(role).or_default().push(grantee);
        self
    }

    pub fn staging_duration(mut self, staging_duration: Duration) -> Self {
        self.staging_duration = Some(staging_duration);
        self
    }

    pub async fn build(self) -> anyhow::Result<Env> {
        let sandbox = Sandbox::new().await?;
        let root = sandbox.root_account().clone();

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

        let s = Env {
            user1: sandbox.create_account("user1").await,
            user2: sandbox.create_account("user2").await,
            user3: sandbox.create_account("user3").await,
            defuse: root
                .deploy_defuse(
                    "defuse",
                    self.fee,
                    self.fee_collector.as_ref().unwrap_or(root.id()),
                    self.super_admins
                        .into_iter()
                        .chain(
                            Some(format!("defuse.{}", root.id()).parse().unwrap())
                                .filter(|_| self.self_as_super_admin),
                        )
                        .chain(Some(root.id().clone()).filter(|_| self.deployer_as_super_admin)),
                    self.admins,
                    self.grantees,
                    self.staging_duration,
                )
                .await?,
            ft1: root
                .poa_factory_deploy_token(poa_factory.id(), "ft1")
                .await?,
            ft2: root
                .poa_factory_deploy_token(poa_factory.id(), "ft2")
                .await?,
            ft3: root
                .poa_factory_deploy_token(poa_factory.id(), "ft3")
                .await?,
            poa_factory,
            sandbox,
        };

        s.ft_storage_deposit(
            &s.ft1,
            &[
                s.user1.id(),
                s.user2.id(),
                s.user3.id(),
                s.defuse.id(),
                root.id(),
            ],
        )
        .await?;
        s.ft_storage_deposit(
            &s.ft2,
            &[
                s.user1.id(),
                s.user2.id(),
                s.user3.id(),
                s.defuse.id(),
                root.id(),
            ],
        )
        .await?;
        s.ft_storage_deposit(
            &s.ft3,
            &[
                s.user1.id(),
                s.user2.id(),
                s.user3.id(),
                s.defuse.id(),
                root.id(),
            ],
        )
        .await?;
        for token in ["ft1", "ft2", "ft3"] {
            s.poa_factory_ft_deposit(
                s.poa_factory.id(),
                token,
                root.id(),
                1_000_000_000,
                None,
                None,
            )
            .await?;
        }

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
        s.user3
            .add_public_key(
                s.defuse.id(),
                s.user3.secret_key().public_key().to_string().parse()?,
            )
            .await?;

        Ok(s)
    }
}
