use near_sdk::AccountId;
use near_workspaces::{Account, Contract};

use crate::utils::{intent::Intending, token::Token, Sandbox};

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
        let token_a = sandbox.deploy_token("token_a").await;
        let token_b = sandbox.deploy_token("token_b").await;
        let intent = sandbox.deploy_intent_contract().await;

        let user = sandbox.create_account("user").await;
        let solver = sandbox.create_account("solver").await;

        intent
            .as_account()
            .add_solver(intent.id(), solver.id())
            .await;

        let solver2 = if self.concurrent_solvers {
            let solver2 = sandbox.create_account("solver2").await;
            intent
                .as_account()
                .add_solver(intent.id(), solver2.id())
                .await;

            if self.storage_deposit {
                token_a.storage_deposit(solver2.id()).await;
                token_b.storage_deposit(solver2.id()).await;
            }

            Some(solver2)
        } else {
            None
        };

        if self.storage_deposit {
            token_a.register_accounts(&[&user, &solver]).await;
            token_b.register_accounts(&[&user, &solver]).await;
        }

        // Transfer tokens to the intent contract to have possibility to refund in case of error in
        // the ft_on_transfer callback.
        token_a.storage_deposit(intent.id()).await;
        token_b.storage_deposit(intent.id()).await;

        if self.fund_intent {
            token_a.ft_transfer(intent.id(), 10_000).await;
            token_b.ft_transfer(intent.id(), 10_000).await;
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
