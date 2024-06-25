use near_sdk::AccountId;
use near_workspaces::{Account, Contract};

use crate::utils::{ft::FtExt, intent::Intending, Sandbox};

use super::SwapFtIntentShard;

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
        let token_a = sandbox.root_account().deploy_ft_token("token_a").await;
        let token_b = sandbox.root_account().deploy_ft_token("token_b").await;
        let intent = sandbox
            .root_account()
            .deploy_swap_ft_intent_shard("ft-intent")
            .await;

        let user = sandbox.create_account("user").await;
        let solver = sandbox.create_account("solver").await;

        intent
            .as_account()
            .add_solver(intent.id(), solver.id())
            .await;
        user.storage_deposit(token_a.id(), None).await;
        user.storage_deposit(token_b.id(), None).await;
        solver.storage_deposit(token_a.id(), None).await;
        solver.storage_deposit(token_b.id(), None).await;
        intent
            .as_account()
            .storage_deposit(token_a.id(), None)
            .await;
        intent
            .as_account()
            .storage_deposit(token_b.id(), None)
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
