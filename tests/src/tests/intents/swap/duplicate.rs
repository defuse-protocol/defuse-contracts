use std::time::{Duration, SystemTime};

use defuse_contracts::intents::swap::{Asset, CreateSwapIntentAction, Deadline};
use near_sdk::NearToken;

use crate::utils::Sandbox;

use super::SwapIntentShard;

#[tokio::test]
async fn test_create_duplicate_native() {
    let sandbox = Sandbox::new().await.unwrap();
    let dao = sandbox
        .create_subaccount("dao", NearToken::from_near(100))
        .await
        .unwrap();
    let swap_intent_shard = dao.deploy_swap_intent_shard("swap-intent").await.unwrap();

    let user = sandbox
        .create_subaccount("user", NearToken::from_near(10))
        .await
        .unwrap();

    assert!(user
        .create_swap_intent(
            swap_intent_shard.id(),
            Asset::Native(NearToken::from_near(3)),
            CreateSwapIntentAction {
                id: "1".to_string(),
                asset_out: Asset::Native(NearToken::from_near(5)),
                recipient: None,
                deadline: Deadline::Timestamp(
                    (SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        + Duration::from_secs(60))
                    .as_secs(),
                ),
            },
        )
        .await
        .unwrap());

    assert!(user
        .create_swap_intent(
            swap_intent_shard.id(),
            Asset::Native(NearToken::from_near(3)),
            CreateSwapIntentAction {
                id: "1".to_string(),
                asset_out: Asset::Native(NearToken::from_near(5)),
                recipient: None,
                deadline: Deadline::Timestamp(
                    (SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        + Duration::from_secs(60))
                    .as_secs(),
                ),
            },
        )
        .await
        .is_err())
}
