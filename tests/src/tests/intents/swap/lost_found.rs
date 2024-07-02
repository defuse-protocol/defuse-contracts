use std::time::{Duration, SystemTime};

use defuse_contracts::intents::swap::{
    Asset, CreateSwapIntentAction, Deadline, ExecuteSwapIntentAction, FtAmount, LostAsset,
    SwapIntentStatus,
};
use near_sdk::NearToken;

use crate::utils::{ft::FtExt, storage_management::StorageManagementExt, Sandbox};

use super::SwapIntentShard;

#[tokio::test]
async fn test_rollback_lost_found_ft() {
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

    user.ft_storage_deposit(ft_token.id(), None).await.unwrap();
    swap_intent_shard
        .ft_storage_deposit(ft_token.id(), None)
        .await
        .unwrap();

    ft_token.ft_mint(user.id(), 1000).await.unwrap();

    let intent_id = "1".to_string();

    assert!(user
        .create_swap_intent(
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
        .await
        .unwrap());

    assert_eq!(ft_token.ft_balance_of(user.id()).await.unwrap(), 0);
    assert_eq!(
        ft_token
            .ft_balance_of(swap_intent_shard.id())
            .await
            .unwrap(),
        1000
    );

    assert!(user.storage_unregister(ft_token.id(), None).await.unwrap());

    assert!(!user
        .rollback_intent(swap_intent_shard.id(), &intent_id)
        .await
        .unwrap());
    assert_eq!(
        swap_intent_shard
            .get_swap_intent(&intent_id)
            .await
            .unwrap()
            .unwrap()
            .as_unlocked()
            .unwrap()
            .as_lost(),
        Some(&LostAsset {
            asset: Asset::Ft(FtAmount {
                token: ft_token.id().clone(),
                amount: 1000
            }),
            recipient: user.id().clone(),
        })
    );
    assert_eq!(ft_token.ft_balance_of(user.id()).await.unwrap(), 0);
    assert_eq!(
        ft_token
            .ft_balance_of(swap_intent_shard.id())
            .await
            .unwrap(),
        1000,
    );

    // try to lost_found before storage_deposit
    assert!(!user
        .lost_found(swap_intent_shard.id(), &intent_id)
        .await
        .unwrap());
    assert_eq!(
        swap_intent_shard
            .get_swap_intent(&intent_id)
            .await
            .unwrap()
            .unwrap()
            .as_unlocked()
            .unwrap()
            .as_lost(),
        Some(&LostAsset {
            asset: Asset::Ft(FtAmount {
                token: ft_token.id().clone(),
                amount: 1000
            }),
            recipient: user.id().clone(),
        })
    );
    assert_eq!(ft_token.ft_balance_of(user.id()).await.unwrap(), 0);
    assert_eq!(
        ft_token
            .ft_balance_of(swap_intent_shard.id())
            .await
            .unwrap(),
        1000,
    );

    user.ft_storage_deposit(ft_token.id(), None).await.unwrap();
    assert!(user
        .lost_found(swap_intent_shard.id(), &intent_id)
        .await
        .unwrap());

    assert_eq!(
        swap_intent_shard.get_swap_intent(&intent_id).await.unwrap(),
        None,
    );

    assert_eq!(ft_token.ft_balance_of(user.id()).await.unwrap(), 1000);
    assert_eq!(
        ft_token
            .ft_balance_of(swap_intent_shard.id())
            .await
            .unwrap(),
        0
    );
}

// TODO: test_rollback_lost_found_nft

#[tokio::test]
async fn test_fulfill_lost_found_ft() {
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

    let user1 = sandbox
        .create_subaccount("user1", NearToken::from_near(10))
        .await
        .unwrap();
    let user2 = sandbox
        .create_subaccount("user2", NearToken::from_near(10))
        .await
        .unwrap();

    user1.ft_storage_deposit(ft_token.id(), None).await.unwrap();
    swap_intent_shard
        .ft_storage_deposit(ft_token.id(), None)
        .await
        .unwrap();

    ft_token.ft_mint(user1.id(), 500).await.unwrap();

    assert_eq!(ft_token.ft_balance_of(user1.id()).await.unwrap(), 500);
    assert_eq!(ft_token.ft_balance_of(user2.id()).await.unwrap(), 0);

    let intent_id = "1".to_string();
    assert!(user1
        .create_swap_intent(
            swap_intent_shard.id(),
            Asset::Ft(FtAmount {
                token: ft_token.id().clone(),
                amount: 500,
            }),
            CreateSwapIntentAction {
                id: intent_id.clone(),
                asset_out: Asset::Native(NearToken::from_near(5)),
                recipient: None,
                deadline: Deadline::Timestamp(
                    (SystemTime::now() + Duration::from_secs(60))
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                ),
            },
        )
        .await
        .unwrap());

    assert_eq!(ft_token.ft_balance_of(user1.id()).await.unwrap(), 0);
    assert_eq!(
        ft_token
            .ft_balance_of(swap_intent_shard.id())
            .await
            .unwrap(),
        500
    );

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
        .unwrap());

    assert_eq!(
        swap_intent_shard
            .get_swap_intent(&intent_id)
            .await
            .unwrap()
            .unwrap()
            .as_unlocked(),
        Some(&SwapIntentStatus::Lost(LostAsset {
            asset: Asset::Ft(FtAmount {
                token: ft_token.id().clone(),
                amount: 500,
            }),
            recipient: user2.id().clone(),
        })),
    );
    assert!(user1.view_account().await.unwrap().balance > NearToken::from_near(14));
    assert_eq!(
        ft_token
            .ft_balance_of(swap_intent_shard.id())
            .await
            .unwrap(),
        500
    );
    assert_eq!(ft_token.ft_balance_of(user2.id()).await.unwrap(), 0);
    assert!(user2.view_account().await.unwrap().balance <= NearToken::from_near(5));

    // no storage_deposit yet
    assert!(!user2
        .lost_found(swap_intent_shard.id(), &intent_id)
        .await
        .unwrap());
    assert_eq!(
        ft_token
            .ft_balance_of(swap_intent_shard.id())
            .await
            .unwrap(),
        500
    );
    assert_eq!(ft_token.ft_balance_of(user2.id()).await.unwrap(), 0);

    user2.ft_storage_deposit(ft_token.id(), None).await.unwrap();
    assert!(user2
        .lost_found(swap_intent_shard.id(), &intent_id)
        .await
        .unwrap());

    assert_eq!(
        ft_token
            .ft_balance_of(swap_intent_shard.id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(ft_token.ft_balance_of(user2.id()).await.unwrap(), 500);

    assert_eq!(
        swap_intent_shard.get_swap_intent(&intent_id).await.unwrap(),
        None,
    );
}
