use std::time::{Duration, SystemTime};

use defuse_contracts::intents::swap::{Asset, CreateSwapIntentAction, Deadline, FtAmount};
use near_sdk::NearToken;

use crate::utils::{ft::FtExt, Sandbox};

use super::SwapIntentShard;

#[tokio::test]
async fn test_create_zero_amount_in_native() {
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
            Asset::Native(NearToken::from_near(0)),
            CreateSwapIntentAction {
                id: "1".to_string(),
                asset_out: Asset::Native(NearToken::from_near(1)),
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
        .is_err());

    assert!(user.view_account().await.unwrap().balance > NearToken::from_near(9));
}

#[tokio::test]
async fn test_create_zero_amount_out_native() {
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
            Asset::Native(NearToken::from_near(5)),
            CreateSwapIntentAction {
                id: "1".to_string(),
                asset_out: Asset::Native(NearToken::from_near(0)),
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
        .is_err());

    assert!(user.view_account().await.unwrap().balance > NearToken::from_near(9));
}

#[tokio::test]
async fn test_create_zero_amount_out_ft() {
    let sandbox = Sandbox::new().await.unwrap();
    let ft_token = sandbox
        .root_account()
        .deploy_ft_token("ft-token")
        .await
        .unwrap();
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
            Asset::Native(NearToken::from_near(5)),
            CreateSwapIntentAction {
                id: "1".to_string(),
                asset_out: Asset::Ft(FtAmount {
                    token: ft_token.id().clone(),
                    amount: 0,
                }),
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
        .is_err());

    assert!(user.view_account().await.unwrap().balance > NearToken::from_near(9));
}
