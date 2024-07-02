use std::time::{Duration, SystemTime};

use defuse_contracts::intents::swap::{
    Asset, CreateSwapIntentAction, Deadline, ExecuteSwapIntentAction,
};
use near_sdk::NearToken;

use crate::utils::Sandbox;

use super::SwapIntentShard;

#[tokio::test]
async fn test_execute_expired() {
    let sandbox = Sandbox::new().await.unwrap();
    let dao = sandbox
        .create_subaccount("dao", NearToken::from_near(100))
        .await
        .unwrap();
    let swap_intent_shard = dao.deploy_swap_intent_shard("swap-intent").await.unwrap();

    let user1 = sandbox
        .create_subaccount("user1", NearToken::from_near(10))
        .await
        .unwrap();
    let user2 = sandbox
        .create_subaccount("user2", NearToken::from_near(20))
        .await
        .unwrap();

    let intent_id = "1".to_string();

    assert!(user1
        .create_swap_intent(
            swap_intent_shard.id(),
            Asset::Native(NearToken::from_near(3)),
            CreateSwapIntentAction {
                id: intent_id.clone(),
                asset_out: Asset::Native(NearToken::from_near(5)),
                recipient: None,
                deadline: Deadline::Timestamp(
                    (SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        + Duration::from_secs(5))
                    .as_secs(),
                ),
            },
        )
        .await
        .unwrap());

    assert!(swap_intent_shard
        .get_swap_intent(&intent_id)
        .await
        .unwrap()
        .unwrap()
        .as_unlocked()
        .unwrap()
        .is_available());

    assert!(user1.view_account().await.unwrap().balance < NearToken::from_near(7));

    tokio::time::sleep(Duration::from_secs(5)).await;

    assert!(user2
        .execute_swap_intent(
            swap_intent_shard.id(),
            Asset::Native(NearToken::from_near(5)),
            ExecuteSwapIntentAction {
                id: intent_id.clone(),
                recipient: None,
            },
        )
        .await
        .is_err());

    assert!(swap_intent_shard
        .get_swap_intent(&intent_id)
        .await
        .unwrap()
        .unwrap()
        .as_unlocked()
        .unwrap()
        .is_available());

    assert!(user1
        .rollback_intent(swap_intent_shard.id(), &intent_id)
        .await
        .unwrap());

    assert_eq!(
        swap_intent_shard
            .get_swap_intent(&"1".to_string())
            .await
            .unwrap(),
        None
    );
    assert!(user1.view_account().await.unwrap().balance > NearToken::from_near(9));
}
