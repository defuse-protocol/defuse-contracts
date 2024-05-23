use crate::utils::intent::Intent;
use crate::utils::token::Token;
use crate::utils::Sandbox;
use near_sdk::AccountId;
use near_workspaces::{Account, Contract};

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
            intent,
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
}
