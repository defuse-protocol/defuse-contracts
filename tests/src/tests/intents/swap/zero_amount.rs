use std::time::Duration;

use defuse_contracts::intents::swap::{Asset, CreateSwapIntentAction, Expiration, FtAmount};
use near_sdk::NearToken;

use super::{Env, SwapIntentShard};

#[tokio::test]
async fn test_create_zero_amount_in_native() {
    let env = Env::new().await.unwrap();

    assert!(env
        .user1
        .create_swap_intent(
            env.swap_intent.id(),
            Asset::Native(NearToken::from_near(0)),
            CreateSwapIntentAction {
                id: "1".to_string(),
                asset_out: Asset::Native(NearToken::from_near(1)),
                recipient: None,
                expiration: Expiration::timeout(Duration::from_secs(60)),
            },
        )
        .await
        .is_err());

    assert!(env.user1.view_account().await.unwrap().balance > NearToken::from_near(9));
}

#[tokio::test]
async fn test_create_zero_amount_out_native() {
    let env = Env::new().await.unwrap();

    assert!(env
        .user1
        .create_swap_intent(
            env.swap_intent.id(),
            Asset::Native(NearToken::from_near(5)),
            CreateSwapIntentAction {
                id: "1".to_string(),
                asset_out: Asset::Native(NearToken::from_near(0)),
                recipient: None,
                expiration: Expiration::timeout(Duration::from_secs(60)),
            },
        )
        .await
        .is_err());

    assert!(env.user1.view_account().await.unwrap().balance > NearToken::from_near(9));
}

#[tokio::test]
async fn test_create_zero_amount_out_ft() {
    let env = Env::new().await.unwrap();

    assert!(env
        .user1
        .create_swap_intent(
            env.swap_intent.id(),
            Asset::Native(NearToken::from_near(5)),
            CreateSwapIntentAction {
                id: "1".to_string(),
                asset_out: Asset::Ft(FtAmount::new(env.ft1.id().clone(), 0)),
                recipient: None,
                expiration: Expiration::timeout(Duration::from_secs(60)),
            },
        )
        .await
        .is_err());

    assert!(env.user1.view_account().await.unwrap().balance > NearToken::from_near(9));
}
