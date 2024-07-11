use std::time::Duration;

use defuse_contracts::intents::swap::{
    Asset, CreateSwapIntentAction, ExecuteSwapIntentAction, Expiration,
};
use near_sdk::NearToken;

use super::{Env, SwapIntentShard};

#[tokio::test]
async fn test_create_duplicate_native() {
    let env = Env::new().await.unwrap();

    assert!(env
        .user1
        .create_swap_intent(
            env.swap_intent.id(),
            Asset::Native(NearToken::from_near(3)),
            CreateSwapIntentAction {
                id: "1".to_string(),
                asset_out: Asset::Native(NearToken::from_near(5)),
                recipient: None,
                expiration: Expiration::timeout(Duration::from_secs(60)),
            },
        )
        .await
        .unwrap());

    assert!(env
        .user1
        .create_swap_intent(
            env.swap_intent.id(),
            Asset::Native(NearToken::from_near(3)),
            CreateSwapIntentAction {
                id: "1".to_string(),
                asset_out: Asset::Native(NearToken::from_near(5)),
                recipient: None,
                expiration: Expiration::timeout(Duration::from_secs(60)),
            },
        )
        .await
        .is_err());
}

#[tokio::test]
async fn test_execute_duplicate_native() {
    let env = Env::new().await.unwrap();

    assert!(env
        .user1
        .create_swap_intent(
            env.swap_intent.id(),
            Asset::Native(NearToken::from_near(3)),
            CreateSwapIntentAction {
                id: "1".to_string(),
                asset_out: Asset::Native(NearToken::from_near(5)),
                recipient: None,
                expiration: Expiration::timeout(Duration::from_secs(60)),
            },
        )
        .await
        .unwrap());

    assert!(env
        .user2
        .execute_swap_intent(
            env.swap_intent.id(),
            Asset::Native(NearToken::from_near(3)),
            ExecuteSwapIntentAction {
                id: "1".to_string(),
                recipient: None,
            },
        )
        .await
        .is_err());
}
