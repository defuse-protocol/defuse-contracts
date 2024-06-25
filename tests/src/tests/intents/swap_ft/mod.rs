mod env;

use defuse_contracts::intents::swap_ft::{Expiration, Intent, Status, TokenAmount};
use lazy_static::lazy_static;
use near_workspaces::Contract;
use serde_json::json;

use crate::utils::{account::AccountExt, ft::FtExt, intent::Intending, read_wasm};

use self::env::Env;

lazy_static! {
    static ref SWAP_FT_INTENT_WASM: Vec<u8> = read_wasm("defuse-swap-ft-intent-contract");
}

pub trait SwapFtIntentShard {
    async fn deploy_swap_ft_intent_shard(
        &self,
        swap_ft_intent_shard_id: impl AsRef<str>,
    ) -> Contract;
}

impl SwapFtIntentShard for near_workspaces::Account {
    async fn deploy_swap_ft_intent_shard(
        &self,
        swap_ft_intent_shard_id: impl AsRef<str>,
    ) -> Contract {
        let contract = self
            .deploy_contract(swap_ft_intent_shard_id, &SWAP_FT_INTENT_WASM)
            .await;

        contract
            .call("new")
            .args_json(json!({
                "owner_id": contract.id()
            }))
            .max_gas()
            .transact()
            .await
            .unwrap()
            .into_result()
            .unwrap();

        contract
    }
}

#[tokio::test]
async fn test_generic_successful_flow() {
    let env = Env::create().await;

    // Deposit 1000 TokenA to the user and 2000 TokenB to the solver.
    env.token_a
        .as_account()
        .ft_transfer(env.token_a.id(), env.user_id(), 1000, None)
        .await;
    env.token_b
        .as_account()
        .ft_transfer(env.token_b.id(), env.solver_id(), 2000, None)
        .await;

    // Check that the user doesn't have TokenB and the solver TokenA.
    assert_eq!(
        env.token_a.as_account().ft_balance_of(env.user_id()).await,
        1000,
    );
    assert_eq!(
        env.token_b.as_account().ft_balance_of(env.user_id()).await,
        0
    );

    assert_eq!(
        env.token_a
            .as_account()
            .ft_balance_of(env.solver_id())
            .await,
        0
    );
    assert_eq!(
        env.token_b
            .as_account()
            .ft_balance_of(env.solver_id())
            .await,
        2000
    );

    // User creates intent for swapping 1000 TokenA to 2000 TokenB.
    env.user
        .create_intent(
            env.token_a.id(),
            env.ft_intent.id(),
            "1",
            Intent {
                initiator: env.user_id().clone(),
                send: TokenAmount {
                    token_id: env.token_a.id().clone(),
                    amount: 1000.into(),
                },
                receive: TokenAmount {
                    token_id: env.token_b.id().clone(),
                    amount: 2000.into(),
                },
                expiration: Expiration::default(),
                referral: None,
            },
        )
        .await;

    // Check that intent contract owns user's TokenA.
    assert_eq!(
        env.token_a.as_account().ft_balance_of(env.user_id()).await,
        0
    );
    assert_eq!(
        env.token_a
            .as_account()
            .ft_balance_of(env.ft_intent.id())
            .await,
        1000
    );

    // The solver is happy with such intent and executes it.
    env.solver
        .execute_intent(env.token_b.id(), env.ft_intent.id(), "1", 2000.into())
        .await;

    // Check balances after intent execution.
    assert_eq!(
        env.token_a.as_account().ft_balance_of(env.user_id()).await,
        0
    );
    assert_eq!(
        env.token_b.as_account().ft_balance_of(env.user_id()).await,
        2000
    );

    assert_eq!(
        env.token_a
            .as_account()
            .ft_balance_of(env.solver_id())
            .await,
        1000
    );
    assert_eq!(
        env.token_b
            .as_account()
            .ft_balance_of(env.solver_id())
            .await,
        0
    );

    // Check that intent has been removed form the state.
    let intent = env.user.get_intent(env.ft_intent.id(), "1").await.unwrap();
    assert!(matches!(intent.status(), Status::Completed));
}

#[tokio::test]
async fn test_successful_flow_partly() {
    let env = Env::create().await;

    // Deposit 1000 TokenA to the user and 2000 TokenB to the solver.
    env.token_a
        .as_account()
        .ft_transfer(env.token_a.id(), env.user_id(), 1000, None)
        .await;
    env.token_b
        .as_account()
        .ft_transfer(env.token_b.id(), env.solver_id(), 2000, None)
        .await;

    // Check that the user doesn't have TokenB and the solver TokenA.
    assert_eq!(
        env.token_a.as_account().ft_balance_of(env.user_id()).await,
        1000
    );
    assert_eq!(
        env.token_b.as_account().ft_balance_of(env.user_id()).await,
        0
    );

    assert_eq!(
        env.token_a
            .as_account()
            .ft_balance_of(env.solver_id())
            .await,
        0
    );
    assert_eq!(
        env.token_b
            .as_account()
            .ft_balance_of(env.solver_id())
            .await,
        2000
    );

    // User creates intent for swapping 1000 TokenA to 2000 TokenB.
    env.user
        .create_intent(
            env.token_a.id(),
            env.ft_intent.id(),
            "1",
            Intent {
                initiator: env.user_id().clone(),
                send: TokenAmount {
                    token_id: env.token_a.id().clone(),
                    amount: 500.into(),
                },
                receive: TokenAmount {
                    token_id: env.token_b.id().clone(),
                    amount: 1000.into(),
                },
                expiration: Expiration::default(),
                referral: None,
            },
        )
        .await;

    // Check that intent contract owns user's TokenA.
    assert_eq!(
        env.token_a.as_account().ft_balance_of(env.user_id()).await,
        500
    );
    assert_eq!(
        env.token_a
            .as_account()
            .ft_balance_of(env.ft_intent.id())
            .await,
        500
    );

    // The solver is happy with such intent and executes it.
    env.solver
        .execute_intent(env.token_b.id(), env.ft_intent.id(), "1", 1000.into())
        .await;

    // Check balances after intent execution.
    assert_eq!(
        env.token_a.as_account().ft_balance_of(env.user_id()).await,
        500
    );
    assert_eq!(
        env.token_b.as_account().ft_balance_of(env.user_id()).await,
        1000
    );

    assert_eq!(
        env.token_a
            .as_account()
            .ft_balance_of(env.solver_id())
            .await,
        500
    );
    assert_eq!(
        env.token_b
            .as_account()
            .ft_balance_of(env.solver_id())
            .await,
        1000
    );
}

#[tokio::test]
async fn test_execute_non_existed_intent() {
    let env = Env::create().await;

    env.token_a
        .as_account()
        .ft_transfer(env.token_a.id(), env.user_id(), 1000, None)
        .await;
    env.token_b
        .as_account()
        .ft_transfer(env.token_b.id(), env.solver_id(), 2000, None)
        .await;

    assert_eq!(
        env.token_a.as_account().ft_balance_of(env.user_id()).await,
        1000
    );
    assert_eq!(
        env.token_a
            .as_account()
            .ft_balance_of(env.solver_id())
            .await,
        0
    );

    assert_eq!(
        env.token_b.as_account().ft_balance_of(env.user_id()).await,
        0
    );
    assert_eq!(
        env.token_b
            .as_account()
            .ft_balance_of(env.solver_id())
            .await,
        2000
    );

    env.solver
        .execute_intent(env.token_b.id(), env.ft_intent.id(), "1", 2000.into())
        .await;

    assert_eq!(
        env.token_a.as_account().ft_balance_of(env.user_id()).await,
        1000
    );
    assert_eq!(
        env.token_a
            .as_account()
            .ft_balance_of(env.solver_id())
            .await,
        0
    );

    assert_eq!(
        env.token_b.as_account().ft_balance_of(env.user_id()).await,
        0
    );
    assert_eq!(
        env.token_b
            .as_account()
            .ft_balance_of(env.solver_id())
            .await,
        2000
    );
}

#[tokio::test]
async fn test_rollback_intent() {
    let env = Env::create().await;

    // Deposit 1000 TokenA to the user and 2000 TokenB to the solver.
    env.token_a
        .as_account()
        .ft_transfer(env.token_a.id(), env.user_id(), 1000, None)
        .await;
    env.token_b
        .as_account()
        .ft_transfer(env.token_b.id(), env.solver_id(), 2000, None)
        .await;

    // Check that the user doesn't have TokenB and the solver TokenA.
    assert_eq!(
        env.token_a.as_account().ft_balance_of(env.user_id()).await,
        1000
    );
    assert_eq!(
        env.token_b.as_account().ft_balance_of(env.user_id()).await,
        0
    );

    assert_eq!(
        env.token_a
            .as_account()
            .ft_balance_of(env.solver_id())
            .await,
        0
    );
    assert_eq!(
        env.token_b
            .as_account()
            .ft_balance_of(env.solver_id())
            .await,
        2000
    );

    // Decrease TTL to 1 second. (Default 5 min).
    env.set_min_ttl(1).await;

    // User creates intent for swapping 1000 TokenA to 2000 TokenB.
    env.user
        .create_intent(
            env.token_a.id(),
            env.ft_intent.id(),
            "1",
            Intent {
                initiator: env.user_id().clone(),
                send: TokenAmount {
                    token_id: env.token_a.id().clone(),
                    amount: 1000.into(),
                },
                receive: TokenAmount {
                    token_id: env.token_b.id().clone(),
                    amount: 2000.into(),
                },
                expiration: Expiration::default(),
                referral: None,
            },
        )
        .await;

    // Check that intent contract owns user's TokenA now.
    assert_eq!(
        env.token_a.as_account().ft_balance_of(env.user_id()).await,
        0
    );
    assert_eq!(
        env.token_a
            .as_account()
            .ft_balance_of(env.ft_intent.id())
            .await,
        1000
    );

    let intent = env.user.get_intent(env.ft_intent.id(), "1").await.unwrap();
    assert!(matches!(intent.status(), Status::Available));

    // The user decides to roll back intent.
    let status = env.user.rollback_intent(env.ft_intent.id(), "1").await;
    assert!(status.is_success());

    let intent = env.user.get_intent(env.ft_intent.id(), "1").await.unwrap();
    assert!(matches!(intent.status(), Status::RolledBack));

    // Check balances after intent execution.
    assert_eq!(
        env.token_a.as_account().ft_balance_of(env.user_id()).await,
        1000
    );
    assert_eq!(
        env.token_a
            .as_account()
            .ft_balance_of(env.solver_id())
            .await,
        0
    );

    assert_eq!(
        env.token_b.as_account().ft_balance_of(env.user_id()).await,
        0
    );
    assert_eq!(
        env.token_b
            .as_account()
            .ft_balance_of(env.solver_id())
            .await,
        2000
    );
}

#[tokio::test]
async fn test_rollback_intent_too_early() {
    let env = Env::create().await;

    // Deposit 1000 TokenA to the user and 2000 TokenB to the solver.
    env.token_a
        .as_account()
        .ft_transfer(env.token_a.id(), env.user_id(), 1000, None)
        .await;
    env.token_b
        .as_account()
        .ft_transfer(env.token_b.id(), env.solver_id(), 2000, None)
        .await;

    // Check that the user doesn't have TokenB and the solver TokenA.
    assert_eq!(
        env.token_a.as_account().ft_balance_of(env.user_id()).await,
        1000
    );
    assert_eq!(
        env.token_b.as_account().ft_balance_of(env.user_id()).await,
        0
    );

    assert_eq!(
        env.token_a
            .as_account()
            .ft_balance_of(env.solver_id())
            .await,
        0
    );
    assert_eq!(
        env.token_b
            .as_account()
            .ft_balance_of(env.solver_id())
            .await,
        2000
    );

    // User creates intent for swapping 1000 TokenA to 2000 TokenB.
    env.user
        .create_intent(
            env.token_a.id(),
            env.ft_intent.id(),
            "1",
            Intent {
                initiator: env.user_id().clone(),
                send: TokenAmount {
                    token_id: env.token_a.id().clone(),
                    amount: 1000.into(),
                },
                receive: TokenAmount {
                    token_id: env.token_b.id().clone(),
                    amount: 2000.into(),
                },
                expiration: Expiration::default(),
                referral: None,
            },
        )
        .await;

    // Check that intent contract owns user's TokenA now.
    assert_eq!(
        env.token_a.as_account().ft_balance_of(env.user_id()).await,
        0
    );
    assert_eq!(
        env.token_a
            .as_account()
            .ft_balance_of(env.ft_intent.id())
            .await,
        1000
    );

    let intent = env.user.get_intent(env.ft_intent.id(), "1").await;
    assert!(intent.is_some());

    // The user decides to roll back intent too early.
    let result = env.user.rollback_intent(env.ft_intent.id(), "1").await;
    assert!(result.is_failure());

    let intent = env.user.get_intent(env.ft_intent.id(), "1").await.unwrap();
    assert!(matches!(intent.status(), Status::Available));

    // Check balances after intent execution.
    assert_eq!(
        env.token_a.as_account().ft_balance_of(env.user_id()).await,
        0
    );
    assert_eq!(
        env.token_b.as_account().ft_balance_of(env.user_id()).await,
        0
    );

    // User's tokens should be still locked in the intent contract.
    assert_eq!(
        env.token_a
            .as_account()
            .ft_balance_of(env.ft_intent.id())
            .await,
        1000
    );

    assert_eq!(
        env.token_a
            .as_account()
            .ft_balance_of(env.solver_id())
            .await,
        0
    );
    assert_eq!(
        env.token_b
            .as_account()
            .ft_balance_of(env.solver_id())
            .await,
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

async fn test_expired_intent(past: Expiration, future: Expiration) {
    let env = Env::create().await;

    // Deposit 1000 TokenA to the user and 2000 TokenB to the solver.
    env.token_a
        .as_account()
        .ft_transfer(env.token_a.id(), env.user_id(), 1000, None)
        .await;
    env.token_b
        .as_account()
        .ft_transfer(env.token_b.id(), env.solver_id(), 2000, None)
        .await;

    // Check that the user doesn't have TokenB and the solver TokenA.
    assert_eq!(
        env.token_a.as_account().ft_balance_of(env.user_id()).await,
        1000
    );
    assert_eq!(
        env.token_b.as_account().ft_balance_of(env.user_id()).await,
        0
    );

    assert_eq!(
        env.token_a
            .as_account()
            .ft_balance_of(env.solver_id())
            .await,
        0
    );
    assert_eq!(
        env.token_b
            .as_account()
            .ft_balance_of(env.solver_id())
            .await,
        2000
    );

    // User creates an intent which is already expired and shouldn't be executed.
    env.user
        .create_intent(
            env.token_a.id(),
            env.ft_intent.id(),
            "1",
            Intent {
                initiator: env.user_id().clone(),
                send: TokenAmount {
                    token_id: env.token_a.id().clone(),
                    amount: 1000.into(),
                },
                receive: TokenAmount {
                    token_id: env.token_b.id().clone(),
                    amount: 2000.into(),
                },
                expiration: past,
                referral: None,
            },
        )
        .await;

    // Check that intent contract owns user's TokenA now.
    assert_eq!(
        env.token_a.as_account().ft_balance_of(env.user_id()).await,
        0
    );
    assert_eq!(
        env.token_a
            .as_account()
            .ft_balance_of(env.ft_intent.id())
            .await,
        1000
    );

    let intent = env.user.get_intent(env.ft_intent.id(), "1").await;
    assert!(intent.is_some());

    // The solver is happy with such intent and executes it.
    env.solver
        .execute_intent(env.token_b.id(), env.ft_intent.id(), "1", 2000.into())
        .await;

    let intent = env.user.get_intent(env.ft_intent.id(), "1").await.unwrap();
    assert!(matches!(intent.status(), Status::Expired));

    // Check balances after intent execution.
    assert_eq!(
        env.token_a.as_account().ft_balance_of(env.user_id()).await,
        1000
    );
    assert_eq!(
        env.token_a
            .as_account()
            .ft_balance_of(env.solver_id())
            .await,
        0
    );

    assert_eq!(
        env.token_b.as_account().ft_balance_of(env.user_id()).await,
        0
    );
    assert_eq!(
        env.token_b
            .as_account()
            .ft_balance_of(env.solver_id())
            .await,
        2000
    );

    // User creates an intent which will be expired in the future and should be executed.
    env.user
        .create_intent(
            env.token_a.id(),
            env.ft_intent.id(),
            "2",
            Intent {
                initiator: env.user_id().clone(),
                send: TokenAmount {
                    token_id: env.token_a.id().clone(),
                    amount: 1000.into(),
                },
                receive: TokenAmount {
                    token_id: env.token_b.id().clone(),
                    amount: 2000.into(),
                },
                expiration: future,
                referral: None,
            },
        )
        .await;

    // The solver is happy with such intent and executes it.
    env.solver
        .execute_intent(env.token_b.id(), env.ft_intent.id(), "2", 2000.into())
        .await;

    // Check balances after intent execution.
    assert_eq!(
        env.token_a.as_account().ft_balance_of(env.user_id()).await,
        0
    );
    assert_eq!(
        env.token_b.as_account().ft_balance_of(env.user_id()).await,
        2000
    );

    assert_eq!(
        env.token_a
            .as_account()
            .ft_balance_of(env.solver_id())
            .await,
        1000
    );
    assert_eq!(
        env.token_b
            .as_account()
            .ft_balance_of(env.solver_id())
            .await,
        0
    );
}
