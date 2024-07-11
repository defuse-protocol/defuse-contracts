use std::time::Duration;

use defuse_contracts::intents::swap::{
    Asset, CreateSwapIntentAction, ExecuteSwapIntentAction, Expiration,
};
use near_sdk::NearToken;

use super::{Env, SwapIntentShard};

#[tokio::test]
async fn test_execute_expired() {
    let env = Env::new().await.unwrap();

    let intent_id = "1".to_string();

    assert!(env
        .user1
        .create_swap_intent(
            env.swap_intent.id(),
            Asset::Native(NearToken::from_near(3)),
            CreateSwapIntentAction {
                id: intent_id.clone(),
                asset_out: Asset::Native(NearToken::from_near(5)),
                recipient: None,
                expiration: Expiration::timeout(Duration::from_secs(5)),
            },
        )
        .await
        .unwrap());

    assert!(env
        .swap_intent
        .get_swap_intent(&intent_id)
        .await
        .unwrap()
        .unwrap()
        .as_unlocked()
        .unwrap()
        .is_available());

    assert!(env.user1.view_account().await.unwrap().balance < NearToken::from_near(7));

    tokio::time::sleep(Duration::from_secs(5)).await;

    assert!(env
        .user2
        .execute_swap_intent(
            env.swap_intent.id(),
            Asset::Native(NearToken::from_near(5)),
            ExecuteSwapIntentAction {
                id: intent_id.clone(),
                recipient: None,
            },
        )
        .await
        .is_err());

    assert!(env
        .swap_intent
        .get_swap_intent(&intent_id)
        .await
        .unwrap()
        .unwrap()
        .as_unlocked()
        .unwrap()
        .is_available());

    assert!(env
        .user1
        .rollback_intent(env.swap_intent.id(), &intent_id)
        .await
        .unwrap());

    assert_eq!(
        env.swap_intent
            .get_swap_intent(&"1".to_string())
            .await
            .unwrap(),
        None
    );
    assert!(env.user1.view_account().await.unwrap().balance > NearToken::from_near(9));
}
