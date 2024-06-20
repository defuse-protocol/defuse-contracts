use near_sdk::AccountId;
use near_workspaces::{Account, Contract};

use crate::utils::{intent::Intending, token::Token, Sandbox};

pub struct Env {
    pub sandbox: Sandbox,
    pub token_a: Contract,
    pub token_b: Contract,
    pub ft_intent: Contract,
    pub user: Account,
    pub solver: Account,
}

impl Env {
    pub async fn create() -> Self {
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

        token_a
            .register_accounts(&[&user, &solver, intent.as_account()])
            .await;
        token_b
            .register_accounts(&[&user, &solver, intent.as_account()])
            .await;

        Self {
            sandbox,
            token_a,
            token_b,
            ft_intent: intent,
            user,
            solver,
        }
    }

    pub fn user_id(&self) -> &AccountId {
        self.user.id()
    }

    pub fn solver_id(&self) -> &AccountId {
        self.solver.id()
    }

    pub async fn set_min_ttl(&self, min_ttl: u64) {
        self.ft_intent
            .as_account()
            .set_min_ttl(self.ft_intent.id(), min_ttl)
            .await;
    }
}
