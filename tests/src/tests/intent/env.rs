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
}

impl Env {
    pub async fn create() -> Self {
        EnvBuilder::new().with_storage_deposit().build().await
    }

    pub fn user_id(&self) -> &AccountId {
        self.user.id()
    }

    pub fn solver_id(&self) -> &AccountId {
        self.solver.id()
    }

    pub async fn set_min_ttl(&self, min_ttl: u64) {
        self.intent
            .as_account()
            .set_min_ttl(self.intent.id(), min_ttl)
            .await;
    }
}

pub struct EnvBuilder {
    with_storage_deposit: bool,
}

impl EnvBuilder {
    pub const fn new() -> Self {
        Self {
            with_storage_deposit: false,
        }
    }

    pub const fn with_storage_deposit(mut self) -> Self {
        self.with_storage_deposit = true;
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

        if self.with_storage_deposit {
            token_a.register_accounts(&[&user, &solver]).await;
            token_b.register_accounts(&[&user, &solver]).await;
        }

        // Transfer tokens to the intent contract to have possibility to refund in case of error in
        // the ft_on_transfer callback.
        token_a.storage_deposit(intent.id()).await;
        token_b.storage_deposit(intent.id()).await;
        token_a.ft_transfer(intent.id(), 10_000).await;
        token_b.ft_transfer(intent.id(), 10_000).await;

        Env {
            sandbox,
            token_a,
            token_b,
            intent,
            user,
            solver,
        }
    }
}
