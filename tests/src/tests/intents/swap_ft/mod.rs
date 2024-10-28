mod env;
mod ext;

use defuse_contracts::intents::swap_ft::{Expiration, Intent, Status, TokenAmount};
use near_sdk::Gas;

use crate::utils::ft::FtExt;

use self::{
    env::{Env, EnvBuilder},
    ext::SwapFtIntentExt,
};

#[tokio::test]
async fn test_generic_successful_flow() {
    let env = Env::create().await;

    // Deposit 1000 TokenA to the user and 2000 TokenB to the solver.
    env.ft_mint(&env.token_a, env.user_id(), 1000)
        .await
        .unwrap();
    env.ft_mint(&env.token_b, env.solver_id(), 2000)
        .await
        .unwrap();

    // User creates intent for swapping 1000 TokenA to 2000 TokenB.
    create_intent(&env, "1", 1000, 2000, Expiration::default()).await;

    // Check that intent contract owns user's TokenA.
    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.user_id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.intent.id())
            .await
            .unwrap(),
        11000
    );

    // The solver is happy with such intent and executes it.
    env.solver
        .execute_intent(&env.token_b, env.intent.id(), "1", 2000.into())
        .await;

    // Check balances after intent execution.
    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.user_id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_b, env.user_id())
            .await
            .unwrap(),
        2000
    );

    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.solver_id())
            .await
            .unwrap(),
        1000
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_b, env.solver_id())
            .await
            .unwrap(),
        0
    );

    // Check that intent has been removed form the state.
    let intent = env.user.get_intent(env.intent.id(), "1").await.unwrap();
    assert!(matches!(intent.status(), Status::Completed));
}

#[tokio::test]
async fn test_successful_flow_partly() {
    let env = Env::create().await;

    // Deposit 1000 TokenA to the user and 2000 TokenB to the solver.
    env.ft_mint(&env.token_a, env.user_id(), 1000)
        .await
        .unwrap();
    env.ft_mint(&env.token_b, env.solver_id(), 2000)
        .await
        .unwrap();

    // User creates intent for swapping 500 TokenA to 1000 TokenB.
    create_intent(&env, "1", 500, 1000, Expiration::default()).await;

    // Check that intent contract owns user's TokenA.
    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.user_id())
            .await
            .unwrap(),
        500
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.intent.id())
            .await
            .unwrap(),
        10500
    );

    // The solver is happy with such intent and executes it.
    env.solver
        .execute_intent(&env.token_b, env.intent.id(), "1", 1000.into())
        .await;

    // Check balances after intent execution.
    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.user_id())
            .await
            .unwrap(),
        500
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_b, env.user_id())
            .await
            .unwrap(),
        1000
    );

    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.solver_id())
            .await
            .unwrap(),
        500
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_b, env.solver_id())
            .await
            .unwrap(),
        1000
    );
}

#[tokio::test]
async fn test_execute_non_existed_intent() {
    let env = Env::create().await;

    env.ft_mint(&env.token_a, env.user_id(), 1000)
        .await
        .unwrap();
    env.ft_mint(&env.token_b, env.solver_id(), 2000)
        .await
        .unwrap();

    env.solver
        .execute_intent(&env.token_b, env.intent.id(), "1", 2000.into())
        .await;

    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.user_id())
            .await
            .unwrap(),
        1000
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.solver_id())
            .await
            .unwrap(),
        0
    );

    assert_eq!(
        env.ft_token_balance_of(&env.token_b, env.user_id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_b, env.solver_id())
            .await
            .unwrap(),
        2000
    );
}

#[tokio::test]
async fn test_rollback_intent() {
    let env = Env::create().await;

    // Deposit 1000 TokenA to the user and 2000 TokenB to the solver.
    env.ft_mint(&env.token_a, env.user_id(), 1000)
        .await
        .unwrap();
    env.ft_mint(&env.token_b, env.solver_id(), 2000)
        .await
        .unwrap();

    // Decrease TTL to 1 second. (Default 5 min).
    env.set_min_ttl(1).await;

    // User creates intent for swapping 1000 TokenA to 2000 TokenB.
    create_intent(&env, "1", 1000, 2000, Expiration::default()).await;

    // Check that intent contract owns user's TokenA now.
    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.user_id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.intent.id())
            .await
            .unwrap(),
        11000
    );

    let intent = env.user.get_intent(env.intent.id(), "1").await.unwrap();
    assert!(matches!(intent.status(), Status::Available));

    // The user decides to roll back the intent.
    env.user
        .rollback_intent(env.intent.id(), "1")
        .await
        .unwrap();

    let intent = env.user.get_intent(env.intent.id(), "1").await.unwrap();
    assert!(matches!(intent.status(), Status::RolledBack));

    // Check balances after intent execution.
    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.user_id())
            .await
            .unwrap(),
        1000
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.solver_id())
            .await
            .unwrap(),
        0
    );

    assert_eq!(
        env.ft_token_balance_of(&env.token_b, env.user_id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_b, env.solver_id())
            .await
            .unwrap(),
        2000
    );
}

#[tokio::test]
async fn test_rollback_intent_too_early() {
    let env = Env::create().await;

    // Deposit 1000 TokenA to the user and 2000 TokenB to the solver.
    env.ft_mint(&env.token_a, env.user_id(), 1000)
        .await
        .unwrap();
    env.ft_mint(&env.token_b, env.solver_id(), 2000)
        .await
        .unwrap();

    // User creates intent for swapping 1000 TokenA to 2000 TokenB.
    create_intent(&env, "1", 1000, 2000, Expiration::default()).await;

    // Check that intent contract owns user's TokenA now.
    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.user_id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.intent.id())
            .await
            .unwrap(),
        11000
    );

    let intent = env.user.get_intent(env.intent.id(), "1").await;
    assert!(intent.is_some());

    // The user decides to roll back intent too early.
    env.user
        .rollback_intent(env.intent.id(), "1")
        .await
        .unwrap_err();

    let intent = env.user.get_intent(env.intent.id(), "1").await.unwrap();
    assert!(matches!(intent.status(), Status::Available));

    // Check balances after intent execution.
    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.user_id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_b, env.user_id())
            .await
            .unwrap(),
        0
    );

    // User's tokens should be still locked in the intent contract.
    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.intent.id())
            .await
            .unwrap(),
        11000
    );

    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.solver_id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_b, env.solver_id())
            .await
            .unwrap(),
        2000
    );
}

#[tokio::test]
async fn test_block_expired_intent() {
    test_expired_intent(Expiration::Block(1), Expiration::Block(1000)).await;
}

#[tokio::test]
async fn test_time_expired_intent() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    test_expired_intent(Expiration::Time(now - 120), Expiration::Time(now + 120)).await;
}

#[tokio::test]
async fn test_intent_without_initiator_storage_deposit() {
    let env = EnvBuilder::new().build().await;

    // Storage deposit for user on token_a and for solver and intent on both.
    env.user
        .ft_storage_deposit(&env.token_a, None)
        .await
        .unwrap();
    env.solver
        .ft_storage_deposit(&env.token_a, None)
        .await
        .unwrap();
    env.solver
        .ft_storage_deposit(&env.token_b, None)
        .await
        .unwrap();

    // Deposit 1000 TokenA to the user and 2000 TokenB to the solver.
    env.ft_mint(&env.token_a, env.user_id(), 1000)
        .await
        .unwrap();
    env.ft_mint(&env.token_b, env.solver_id(), 2000)
        .await
        .unwrap();

    // Initiator creates intent for swapping 1000 TokenA to 2000 TokenB.
    create_intent(&env, "1", 1000, 2000, Expiration::default()).await;

    let result = env.user.get_intent(env.intent.id(), "1").await;
    assert!(result.is_none()); // No intent because the initiator has no storage deposit.

    // Check that the balances haven't been changed.
    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.user_id())
            .await
            .unwrap(),
        1000
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_b, env.user_id())
            .await
            .unwrap(),
        0
    );

    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.solver_id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_b, env.solver_id())
            .await
            .unwrap(),
        2000
    );
}

#[tokio::test]
async fn test_intent_without_solver_storage_deposit() {
    let env = EnvBuilder::new().build().await;

    // Storage deposit for the initiator and intent on both tokens and for solver on token_b only.
    env.user
        .ft_storage_deposit(&env.token_a, None)
        .await
        .unwrap();
    env.user
        .ft_storage_deposit(&env.token_b, None)
        .await
        .unwrap();
    env.solver
        .ft_storage_deposit(&env.token_b, None)
        .await
        .unwrap();

    // Deposit 1000 TokenA to the user and 2000 TokenB to the solver.
    env.ft_mint(&env.token_a, env.user_id(), 1000)
        .await
        .unwrap();
    env.ft_mint(&env.token_b, env.solver_id(), 2000)
        .await
        .unwrap();

    // User creates intent for swapping 1000 TokenA to 2000 TokenB.
    create_intent(&env, "1", 1000, 2000, Expiration::default()).await;

    // The solver is trying to execute it, but he has no storage deposit.
    env.solver
        .execute_intent(&env.token_b, env.intent.id(), "1", 2000.into())
        .await;

    let result = env.user.get_intent(env.intent.id(), "1").await.unwrap();
    assert!(matches!(result.status(), Status::Available));

    // Check that the balances haven't been changed.
    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.user_id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.intent.id())
            .await
            .unwrap(),
        1000
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_b, env.user_id())
            .await
            .unwrap(),
        0
    );

    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.solver_id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_b, env.solver_id())
            .await
            .unwrap(),
        2000
    );
}

#[ignore = "near-plugins fungible token impl requires too much gas for ft_transfer_call, so it doesn't fail"]
#[tokio::test]
async fn test_intent_with_lack_of_gas_for_creation() {
    let env = Env::create().await;

    // Deposit 1000 TokenA to the user and 2000 TokenB to the solver.
    env.ft_mint(&env.token_a, env.user_id(), 1000)
        .await
        .unwrap();
    env.ft_mint(&env.token_b, env.solver_id(), 2000)
        .await
        .unwrap();

    // User creates intent for swapping 1000 TokenA to 2000 TokenB.
    create_intent_with_gas(
        &env,
        "1",
        1000,
        2000,
        Expiration::default(),
        Gas::from_tgas(35), // 35 TGas is not enough for the creation.
    )
    .await;

    let result = env.user.get_intent(env.intent.id(), "1").await;
    assert_eq!(result, None); // No intent because not enough gas was provided.

    // Check that the balances haven't been changed.
    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.user_id())
            .await
            .unwrap(),
        1000
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_b, env.user_id())
            .await
            .unwrap(),
        0
    );

    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.solver_id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_b, env.solver_id())
            .await
            .unwrap(),
        2000
    );
}

#[tokio::test]
async fn test_intent_with_lack_of_gas_for_execution() {
    let env = Env::create().await;

    // Deposit 1000 TokenA to the user and 2000 TokenB to the solver.
    env.ft_mint(&env.token_a, env.user_id(), 1000)
        .await
        .unwrap();
    env.ft_mint(&env.token_b, env.solver_id(), 2000)
        .await
        .unwrap();

    // User creates intent for swapping 1000 TokenA to 2000 TokenB.
    create_intent(&env, "1", 1000, 2000, Expiration::default()).await;

    // The solver is trying to execute it, but provided not enough gas.
    env.solver
        .execute_intent_with_gas(
            &env.token_b,
            env.intent.id(),
            "1",
            2000.into(),
            Gas::from_tgas(40), // 50 TGas is not enough for the execution.
        )
        .await;

    // Check that the balances haven't been changed.
    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.user_id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.intent.id())
            .await
            .unwrap(),
        11000
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_b, env.user_id())
            .await
            .unwrap(),
        0
    );

    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.solver_id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_b, env.solver_id())
            .await
            .unwrap(),
        2000
    );

    let result = env.user.get_intent(env.intent.id(), "1").await.unwrap();
    assert!(
        matches!(result.status(), Status::Available),
        "{:?}",
        result.status()
    );
}

#[tokio::test]
async fn test_concurrent_solvers() {
    let env = EnvBuilder::new()
        .with_storage_deposit()
        .with_concurrent_solver()
        .build()
        .await;

    // Deposit 1000 TokenA to the user and 2000 TokenB to the solver.
    env.ft_mint(&env.token_a, env.user_id(), 1000)
        .await
        .unwrap();
    env.ft_mint(&env.token_b, env.solver_id(), 2000)
        .await
        .unwrap();
    env.ft_mint(&env.token_b, env.solver2_id(), 2000)
        .await
        .unwrap();

    // User creates intent for swapping 1000 TokenA to 2000 TokenB.
    create_intent(&env, "1", 1000, 2000, Expiration::default()).await;

    // Check that intent contract owns user's TokenA.
    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.user_id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.intent.id())
            .await
            .unwrap(),
        1000
    );

    // The solver is happy with such intent and executes it.
    let result1 = env
        .solver
        .execute_intent_async(&env.token_b, env.intent.id(), "1", 2000.into())
        .await;
    env.sandbox.skip_blocks(1).await;
    // The solver2 is happy with such intent and executes it too.
    let result2 = env
        .solver2
        .as_ref()
        .unwrap()
        .execute_intent_async(&env.token_b, env.intent.id(), "1", 2000.into())
        .await;

    assert!(result1.await.unwrap().is_success());
    assert!(result2.await.unwrap().is_success());

    // Check balances after intent execution.
    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.user_id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_b, env.user_id())
            .await
            .unwrap(),
        2000
    );

    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.solver_id())
            .await
            .unwrap(),
        1000
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_b, env.solver_id())
            .await
            .unwrap(),
        0
    );

    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.solver2_id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_b, env.solver2_id())
            .await
            .unwrap(),
        2000
    );

    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.intent.id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_b, env.intent.id())
            .await
            .unwrap(),
        0
    );

    // Check that intent has been removed form the state.
    let intent = env.user.get_intent(env.intent.id(), "1").await.unwrap();
    assert!(matches!(intent.status(), Status::Completed));
}

async fn test_expired_intent(past: Expiration, future: Expiration) {
    let env = Env::create().await;

    // Deposit 1000 TokenA to the user and 2000 TokenB to the solver.
    env.ft_mint(&env.token_a, env.user_id(), 1000)
        .await
        .unwrap();
    env.ft_mint(&env.token_b, env.solver_id(), 2000)
        .await
        .unwrap();

    // User creates an intent which is already expired and shouldn't be executed.
    create_intent(&env, "1", 1000, 2000, past).await;

    // Check that intent contract owns user's TokenA now.
    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.user_id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.intent.id())
            .await
            .unwrap(),
        11000
    );

    let intent = env.user.get_intent(env.intent.id(), "1").await;
    assert!(intent.is_some());

    // The solver is happy with such intent and executes it.
    env.solver
        .execute_intent(&env.token_b, env.intent.id(), "1", 2000.into())
        .await;

    let intent = env.user.get_intent(env.intent.id(), "1").await.unwrap();
    assert!(matches!(intent.status(), Status::Expired));

    // Check balances after intent execution.
    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.user_id())
            .await
            .unwrap(),
        1000
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.solver_id())
            .await
            .unwrap(),
        0
    );

    assert_eq!(
        env.ft_token_balance_of(&env.token_b, env.user_id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_b, env.solver_id())
            .await
            .unwrap(),
        2000
    );

    // User creates an intent which will be expired in the future and should be executed.
    create_intent(&env, "2", 1000, 2000, future).await;

    // The solver is happy with such intent and executes it.
    env.solver
        .execute_intent(&env.token_b, env.intent.id(), "2", 2000.into())
        .await;

    // Check balances after intent execution.
    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.user_id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_b, env.user_id())
            .await
            .unwrap(),
        2000
    );

    assert_eq!(
        env.ft_token_balance_of(&env.token_a, env.solver_id())
            .await
            .unwrap(),
        1000
    );
    assert_eq!(
        env.ft_token_balance_of(&env.token_b, env.solver_id())
            .await
            .unwrap(),
        0
    );
}

async fn create_intent(env: &Env, id: &str, send: u128, receive: u128, expiration: Expiration) {
    create_intent_with_gas(env, id, send, receive, expiration, Gas::from_tgas(50)).await;
}

async fn create_intent_with_gas(
    env: &Env,
    id: &str,
    send: u128,
    receive: u128,
    expiration: Expiration,
    gas: Gas,
) {
    env.user
        .create_intent_with_gas(
            &env.token_a,
            env.intent.id(),
            id,
            Intent {
                initiator: env.user_id().clone(),
                send: TokenAmount {
                    token_id: env.token_a.clone(),
                    amount: send.into(),
                },
                receive: TokenAmount {
                    token_id: env.token_b.clone(),
                    amount: receive.into(),
                },
                expiration,
                referral: None,
            },
            gas,
        )
        .await;
}
