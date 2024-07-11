use near_sdk::AccountId;
use near_workspaces::{Account, Contract};

use crate::utils::{ft::FtExt, Sandbox};

use super::ext::SwapFtIntentExt;

pub struct Env {
    pub sandbox: Sandbox,
    pub token_a: Contract,
    pub token_b: Contract,
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
        let token_a = sandbox
            .root_account()
            .deploy_ft_token("token_a")
            .await
            .unwrap();
        let token_b = sandbox
            .root_account()
            .deploy_ft_token("token_b")
            .await
            .unwrap();
        let intent = sandbox
            .root_account()
            .deploy_swap_ft_intent_contract()
            .await
            .unwrap();

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
                solver2
                    .ft_storage_deposit(token_a.id(), None)
                    .await
                    .unwrap();
                solver2
                    .ft_storage_deposit(token_b.id(), None)
                    .await
                    .unwrap();
            }

            Some(solver2)
        } else {
            None
        };

        if self.storage_deposit {
            user.ft_storage_deposit(token_a.id(), None).await.unwrap();
            user.ft_storage_deposit(token_b.id(), None).await.unwrap();
            solver.ft_storage_deposit(token_a.id(), None).await.unwrap();
            solver.ft_storage_deposit(token_b.id(), None).await.unwrap();
        }

        // Transfer tokens to the intent contract to have possibility to refund in case of error in
        // the ft_on_transfer callback.
        intent.ft_storage_deposit(token_a.id(), None).await.unwrap();
        intent.ft_storage_deposit(token_b.id(), None).await.unwrap();

        if self.fund_intent {
            token_a.ft_mint(intent.id(), 10_000).await.unwrap();
            token_b.ft_mint(intent.id(), 10_000).await.unwrap();
        }

        Env {
            sandbox,
            token_a,
            token_b,
            intent,
            user,
            solver,
            solver2,
        }
    }
}
