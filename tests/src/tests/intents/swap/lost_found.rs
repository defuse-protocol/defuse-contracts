use std::time::{Duration, SystemTime};

use defuse_contracts::intents::swap::{
    Asset, CreateSwapIntentAction, Deadline, FtAmount, LostFound,
};
use near_sdk::NearToken;

use crate::utils::{ft::FtExt, storage_management::StorageManagementExt, Sandbox};

use super::SwapIntentShard;

#[tokio::test]
async fn test_rollback_lost_found_ft() {
    let sandbox = Sandbox::new().await.unwrap();
    let ft_token = sandbox.root_account().deploy_ft_token("ft-token").await;
    let dao = sandbox
        .create_subaccount("dao", NearToken::from_near(100))
        .await
        .unwrap();
    let swap_intent_shard = dao.deploy_swap_intent_shard("swap-intent").await;
    let user = sandbox
        .create_subaccount("user", NearToken::from_near(10))
        .await
        .unwrap();

    user.ft_storage_deposit(ft_token.id(), None).await;
    swap_intent_shard
        .as_account()
        .ft_storage_deposit(ft_token.id(), None)
        .await;

    ft_token
        .as_account()
        .ft_transfer(ft_token.id(), user.id(), 1000, None)
        .await;
    assert_eq!(ft_token.as_account().ft_balance_of(user.id()).await, 1000);

    let intent_id = "1".to_string();

    user.create_swap_intent(
        swap_intent_shard.id(),
        Asset::Ft(FtAmount {
            token: ft_token.id().clone(),
            amount: 1000,
        }),
        CreateSwapIntentAction {
            id: intent_id.clone(),
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
    .await;

    assert_eq!(ft_token.as_account().ft_balance_of(user.id()).await, 0);
    assert_eq!(
        ft_token
            .as_account()
            .ft_balance_of(swap_intent_shard.id())
            .await,
        1000
    );

    assert!(user.storage_unregister(ft_token.id(), None).await);

    assert!(
        !user
            .rollback_intent(swap_intent_shard.id(), &intent_id)
            .await
    );
    assert_eq!(
        swap_intent_shard
            .as_account()
            .get_swap_intent(&intent_id)
            .await
            .unwrap()
            .get_unlocked()
            .unwrap()
            .as_lost_found(),
        Some(&LostFound {
            asset: Asset::Ft(FtAmount {
                token: ft_token.id().clone(),
                amount: 1000
            }),
            recipient: user.id().clone(),
        })
    );
    assert_eq!(ft_token.as_account().ft_balance_of(user.id()).await, 0);
    assert_eq!(
        ft_token
            .as_account()
            .ft_balance_of(swap_intent_shard.id())
            .await,
        1000,
    );

    // try to lost_found before storage_deposit
    assert!(!user.lost_found(swap_intent_shard.id(), &intent_id).await);
    assert_eq!(
        swap_intent_shard
            .as_account()
            .get_swap_intent(&intent_id)
            .await
            .unwrap()
            .get_unlocked()
            .unwrap()
            .as_lost_found(),
        Some(&LostFound {
            asset: Asset::Ft(FtAmount {
                token: ft_token.id().clone(),
                amount: 1000
            }),
            recipient: user.id().clone(),
        })
    );
    assert_eq!(ft_token.as_account().ft_balance_of(user.id()).await, 0);
    assert_eq!(
        ft_token
            .as_account()
            .ft_balance_of(swap_intent_shard.id())
            .await,
        1000,
    );

    user.ft_storage_deposit(ft_token.id(), None).await;
    assert!(user.lost_found(swap_intent_shard.id(), &intent_id).await);

    assert_eq!(
        swap_intent_shard
            .as_account()
            .get_swap_intent(&intent_id)
            .await,
        None,
    );

    assert_eq!(ft_token.as_account().ft_balance_of(user.id()).await, 1000);
    assert_eq!(
        ft_token
            .as_account()
            .ft_balance_of(swap_intent_shard.id())
            .await,
        0
    );
}

// TODO: test_rollback_lost_found_nft
