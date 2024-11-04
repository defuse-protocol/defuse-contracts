use defuse_poa_factory_contract::Role as POAFactoryRole;
use impl_tools::autoimpl;
use near_sdk::AccountId;
use near_workspaces::{Account, Contract};

use crate::{
    tests::poa::factory::PoAFactoryExt,
    utils::{ft::FtExt, Sandbox},
};

use super::ext::SwapFtIntentExt;

#[autoimpl(Deref using self.sandbox)]
pub struct Env {
    pub sandbox: Sandbox,

    pub poa_factory: Contract,

    pub token_a: AccountId,
    pub token_b: AccountId,
    pub intent: Contract,
    pub user: Account,
    pub solver: Account,
    pub solver2: Option<Account>,
}

impl Env {
    pub async fn create() -> Self {
        EnvBuilder::new()
            .with_storage_deposit()
            .with_fund_intent()
            .build()
            .await
    }

    pub fn user_id(&self) -> &AccountId {
        self.user.id()
    }

    pub fn solver_id(&self) -> &AccountId {
        self.solver.id()
    }

    pub fn solver2_id(&self) -> &AccountId {
        self.solver2
            .as_ref()
            .map(Account::id)
            .expect("Solver2 was not set")
    }

    pub async fn set_min_ttl(&self, min_ttl: u64) {
        self.intent
            .as_account()
            .set_min_ttl(self.intent.id(), min_ttl)
            .await;
    }

    pub async fn ft_mint(
        &self,
        token: &AccountId,
        account_id: &AccountId,
        amount: u128,
    ) -> anyhow::Result<()> {
        self.sandbox
            .root_account()
            .poa_factory_ft_deposit(
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

pub struct EnvBuilder {
    storage_deposit: bool,
    concurrent_solvers: bool,
    fund_intent: bool,
}

impl EnvBuilder {
    pub const fn new() -> Self {
        Self {
            storage_deposit: false,
            concurrent_solvers: false,
            fund_intent: false,
        }
    }

    pub const fn with_storage_deposit(mut self) -> Self {
        self.storage_deposit = true;
        self
    }

    pub const fn with_concurrent_solver(mut self) -> Self {
        self.concurrent_solvers = true;
        self
    }

    pub const fn with_fund_intent(mut self) -> Self {
        self.fund_intent = true;
        self
    }

    pub async fn build(self) -> Env {
        let sandbox = Sandbox::new().await.unwrap();
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
            .await
            .unwrap();

        let token_a = root
            .poa_factory_deploy_token(poa_factory.id(), "token_a", None)
            .await
            .unwrap();
        let token_b = root
            .poa_factory_deploy_token(poa_factory.id(), "token_b", None)
            .await
            .unwrap();
        let intent = root.deploy_swap_ft_intent_contract().await.unwrap();

        let user = sandbox.create_account("user").await;
        let solver = sandbox.create_account("solver").await;

        intent.add_solver(intent.id(), solver.id()).await;

        let solver2 = if self.concurrent_solvers {
            let solver2 = sandbox.create_account("solver2").await;
            intent
                .as_account()
                .add_solver(intent.id(), solver2.id())
                .await;

            if self.storage_deposit {
                solver2.ft_storage_deposit(&token_a, None).await.unwrap();
                solver2.ft_storage_deposit(&token_b, None).await.unwrap();
            }

            Some(solver2)
        } else {
            None
        };

        if self.storage_deposit {
            user.ft_storage_deposit(&token_a, None).await.unwrap();
            user.ft_storage_deposit(&token_b, None).await.unwrap();
            solver.ft_storage_deposit(&token_a, None).await.unwrap();
            solver.ft_storage_deposit(&token_b, None).await.unwrap();
        }

        // Transfer tokens to the intent contract to have possibility to refund in case of error in
        // the ft_on_transfer callback.
        intent.ft_storage_deposit(&token_a, None).await.unwrap();
        intent.ft_storage_deposit(&token_b, None).await.unwrap();

        if self.fund_intent {
            root.poa_factory_ft_deposit(
                poa_factory.id(),
                "token_a",
                intent.id(),
                10_000,
                None,
                None,
            )
            .await
            .unwrap();
            root.poa_factory_ft_deposit(
                poa_factory.id(),
                "token_b",
                intent.id(),
                10_000,
                None,
                None,
            )
            .await
            .unwrap();
        }

        Env {
            sandbox,
            poa_factory,
            token_a,
            token_b,
            intent,
            user,
            solver,
            solver2,
        }
    }
}
