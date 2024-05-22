use defuse_intent_contract::types::{
    intent::nep141::{Nep141, TokenAmount},
    IntentType,
};
use near_workspaces::{types::NearToken, Account};

use crate::utils::{intent::Intent, token::FungibleToken, Sandbox};

// The test demonstrates the following flow:
// 1. User has amount X of tokens A.
// 2. User decides to swap his tokens A to Y amount of tokens B.
// 3. Solver
#[tokio::test]
async fn test_main_generic_intent_flow() {
    let sandbox = Sandbox::new().await.unwrap();
    let token_a = sandbox.deploy_token("token_a").await;
    let token_b = sandbox.deploy_token("token_b").await;
    let user = create_account(&sandbox, "user").await;
    let solver = create_account(&sandbox, "solver").await;

    let intent = sandbox.deploy_intent_contract().await;
    intent.add_solver(solver.id()).await;

    user.create_intent(
        intent.id(),
        "1",
        IntentType::Nep141(Nep141 {
            output: TokenAmount {
                token_id: token_a.id().clone(),
                amount: 1000.into(),
            },
            input: TokenAmount {
                token_id: token_b.id().clone(),
                amount: 2000.into(),
            },
        }),
    )
    .await;

    solver.execute_intent(intent.id(), "1").await;

    assert_eq!(token_a.ft_balance_of(&solver.id()).await, 1000);
    assert_eq!(token_b.ft_balance_of(&user.id()).await, 2000);
}

async fn create_account(sandbox: &Sandbox, name: &str) -> Account {
    sandbox
        .create_subaccount(name, NearToken::from_near(10))
        .await
        .unwrap()
}
