use std::time::Duration;

use defuse_contracts::intents::swap::{
    Asset, CreateSwapIntentAction, ExecuteSwapIntentAction, Expiration, FtAmount, LostAsset,
    SwapIntentStatus,
};
use near_sdk::NearToken;

use crate::utils::{ft::FtExt, storage_management::StorageManagementExt};

use super::{Env, SwapIntentShard};

#[tokio::test]
async fn text_execute_assets_in_out_ft_no_deposits() {
    let env = Env::new().await.unwrap();
    env.root_account()
        .ft_storage_deposit_many(env.ft1.id(), &[env.user1.id(), env.swap_intent.id()])
        .await
        .unwrap();
    env.root_account()
        .ft_storage_deposit_many(env.ft2.id(), &[env.user2.id(), env.swap_intent.id()])
        .await
        .unwrap();

    env.ft1.ft_mint(env.user1.id(), 1000).await.unwrap();
    env.ft2.ft_mint(env.user2.id(), 2000).await.unwrap();

    let intent_id = "1".to_string();

    assert!(env
        .user1
        .create_swap_intent(
            env.swap_intent.id(),
            Asset::Ft(FtAmount {
                token: env.ft1.id().clone(),
                amount: 1000.into(),
            }),
            CreateSwapIntentAction {
                id: intent_id.clone(),
                asset_out: Asset::Ft(FtAmount {
                    token: env.ft2.id().clone(),
                    amount: 2000.into(),
                }),
                recipient: None,
                expiration: Expiration::timeout(Duration::from_secs(60)),
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

    assert_eq!(env.ft1.ft_balance_of(env.user1.id()).await.unwrap(), 0);
    assert_eq!(
        env.ft1.ft_balance_of(env.swap_intent.id()).await.unwrap(),
        1000
    );

    assert!(!env
        .user2
        .execute_swap_intent(
            env.swap_intent.id(),
            Asset::Ft(FtAmount {
                token: env.ft2.id().clone(),
                amount: 2000.into(),
            }),
            ExecuteSwapIntentAction {
                id: intent_id.clone(),
                recipient: None,
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

    env.user1
        .ft_storage_deposit(env.ft2.id(), None)
        .await
        .unwrap();
    env.user2
        .ft_storage_deposit(env.ft1.id(), None)
        .await
        .unwrap();

    assert!(env
        .user2
        .execute_swap_intent(
            env.swap_intent.id(),
            Asset::Ft(FtAmount {
                token: env.ft2.id().clone(),
                amount: 2000.into(),
            }),
            ExecuteSwapIntentAction {
                id: intent_id.clone(),
                recipient: None,
            },
        )
        .await
        .unwrap());

    assert_eq!(
        env.swap_intent.get_swap_intent(&intent_id).await.unwrap(),
        None,
    );

    assert_eq!(env.ft1.ft_balance_of(env.user1.id()).await.unwrap(), 0);
    assert_eq!(env.ft2.ft_balance_of(env.user1.id()).await.unwrap(), 2000);
    assert_eq!(env.ft1.ft_balance_of(env.user2.id()).await.unwrap(), 1000);
    assert_eq!(env.ft2.ft_balance_of(env.user2.id()).await.unwrap(), 0);

    assert_eq!(
        env.ft1.ft_balance_of(env.swap_intent.id()).await.unwrap(),
        0,
    );
    assert_eq!(
        env.ft2.ft_balance_of(env.swap_intent.id()).await.unwrap(),
        0,
    );
}

#[tokio::test]
async fn test_execute_asset_in_ft_no_deposit() {
    let env = Env::new().await.unwrap();
    env.root_account()
        .ft_storage_deposit_many(env.ft1.id(), &[env.user1.id(), env.swap_intent.id()])
        .await
        .unwrap();

    env.ft1.ft_mint(env.user1.id(), 500).await.unwrap();

    let intent_id = "1".to_string();
    assert!(env
        .user1
        .create_swap_intent(
            env.swap_intent.id(),
            Asset::Ft(FtAmount::new(env.ft1.id().clone(), 500)),
            CreateSwapIntentAction {
                id: intent_id.clone(),
                asset_out: Asset::Native(NearToken::from_near(5)),
                recipient: None,
                expiration: Expiration::timeout(Duration::from_secs(60)),
            },
        )
        .await
        .unwrap());

    assert_eq!(env.ft1.ft_balance_of(env.user1.id()).await.unwrap(), 0);
    assert_eq!(
        env.ft1.ft_balance_of(env.swap_intent.id()).await.unwrap(),
        500
    );

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
        .unwrap());

    assert_eq!(
        env.swap_intent
            .get_swap_intent(&intent_id)
            .await
            .unwrap()
            .unwrap()
            .as_unlocked(),
        Some(&SwapIntentStatus::Lost(LostAsset {
            asset: Asset::Ft(FtAmount::new(env.ft1.id().clone(), 500)),
            recipient: env.user2.id().clone(),
        })),
    );
    assert!(env.user1.view_account().await.unwrap().balance > NearToken::from_near(14));
    assert_eq!(
        env.ft1.ft_balance_of(env.swap_intent.id()).await.unwrap(),
        500
    );
    assert_eq!(env.ft1.ft_balance_of(env.user2.id()).await.unwrap(), 0);
    assert!(env.user2.view_account().await.unwrap().balance <= NearToken::from_near(5));

    // no storage_deposit yet
    assert!(!env
        .user2
        .lost_found(env.swap_intent.id(), &intent_id)
        .await
        .unwrap());
    assert_eq!(
        env.ft1.ft_balance_of(env.swap_intent.id()).await.unwrap(),
        500
    );
    assert_eq!(env.ft1.ft_balance_of(env.user2.id()).await.unwrap(), 0);

    env.user2
        .ft_storage_deposit(env.ft1.id(), None)
        .await
        .unwrap();
    assert!(env
        .user2
        .lost_found(env.swap_intent.id(), &intent_id)
        .await
        .unwrap());

    assert_eq!(
        env.ft1.ft_balance_of(env.swap_intent.id()).await.unwrap(),
        0
    );
    assert_eq!(env.ft1.ft_balance_of(env.user2.id()).await.unwrap(), 500);

    assert_eq!(
        env.swap_intent.get_swap_intent(&intent_id).await.unwrap(),
        None,
    );
}

#[tokio::test]
async fn test_execute_asset_out_ft_no_deposit() {
    let env = Env::new().await.unwrap();
    env.root_account()
        .ft_storage_deposit_many(env.ft1.id(), &[env.user2.id(), env.swap_intent.id()])
        .await
        .unwrap();

    env.ft1.ft_mint(env.user2.id(), 500).await.unwrap();

    let intent_id = "1".to_string();
    assert!(env
        .user1
        .create_swap_intent(
            env.swap_intent.id(),
            Asset::Native(NearToken::from_near(5)),
            CreateSwapIntentAction {
                id: intent_id.clone(),
                asset_out: Asset::Ft(FtAmount::new(env.ft1.id().clone(), 500)),
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
            Asset::Ft(FtAmount::new(env.ft1.id().clone(), 500)),
            ExecuteSwapIntentAction {
                id: intent_id.clone(),
                recipient: None,
            },
        )
        .await
        .unwrap());

    assert_eq!(
        env.swap_intent
            .get_swap_intent(&intent_id)
            .await
            .unwrap()
            .unwrap()
            .as_unlocked(),
        Some(&SwapIntentStatus::Lost(LostAsset {
            asset: Asset::Ft(FtAmount::new(env.ft1.id().clone(), 500)),
            recipient: env.user1.id().clone(),
        })),
    );
    assert!(env.user1.view_account().await.unwrap().balance <= NearToken::from_near(5));
    assert_eq!(
        env.ft1.ft_balance_of(env.swap_intent.id()).await.unwrap(),
        500
    );
    assert_eq!(env.ft1.ft_balance_of(env.user1.id()).await.unwrap(), 0);
    assert!(env.user2.view_account().await.unwrap().balance > NearToken::from_near(14));

    // no storage_deposit yet
    assert!(!env
        .user1
        .lost_found(env.swap_intent.id(), &intent_id)
        .await
        .unwrap());
    assert_eq!(
        env.ft1.ft_balance_of(env.swap_intent.id()).await.unwrap(),
        500
    );
    assert_eq!(env.ft1.ft_balance_of(env.user1.id()).await.unwrap(), 0);

    env.user1
        .ft_storage_deposit(env.ft1.id(), None)
        .await
        .unwrap();
    assert!(env
        .user1
        .lost_found(env.swap_intent.id(), &intent_id)
        .await
        .unwrap());

    assert_eq!(
        env.ft1.ft_balance_of(env.swap_intent.id()).await.unwrap(),
        0
    );
    assert_eq!(env.ft1.ft_balance_of(env.user1.id()).await.unwrap(), 500);

    assert_eq!(
        env.swap_intent.get_swap_intent(&intent_id).await.unwrap(),
        None,
    );
}

#[tokio::test]
async fn test_rollback_lost_found_ft() {
    let env = Env::new().await.unwrap();
    env.root_account()
        .ft_storage_deposit_many(env.ft1.id(), &[env.user1.id(), env.swap_intent.id()])
        .await
        .unwrap();

    env.ft1.ft_mint(env.user1.id(), 1000).await.unwrap();

    let intent_id = "1".to_string();

    assert!(env
        .user1
        .create_swap_intent(
            env.swap_intent.id(),
            Asset::Ft(FtAmount::new(env.ft1.id().clone(), 1000)),
            CreateSwapIntentAction {
                id: intent_id.clone(),
                asset_out: Asset::Native(NearToken::from_near(5)),
                recipient: None,
                expiration: Expiration::timeout(Duration::from_secs(60)),
            },
        )
        .await
        .unwrap());

    assert_eq!(env.ft1.ft_balance_of(env.user1.id()).await.unwrap(), 0);
    assert_eq!(
        env.ft1.ft_balance_of(env.swap_intent.id()).await.unwrap(),
        1000
    );

    assert!(env
        .user1
        .storage_unregister(env.ft1.id(), None)
        .await
        .unwrap());

    assert!(!env
        .user1
        .rollback_intent(env.swap_intent.id(), &intent_id)
        .await
        .unwrap());
    assert_eq!(
        env.swap_intent
            .get_swap_intent(&intent_id)
            .await
            .unwrap()
            .unwrap()
            .as_unlocked()
            .unwrap()
            .as_lost(),
        Some(&LostAsset {
            asset: Asset::Ft(FtAmount::new(env.ft1.id().clone(), 1000)),
            recipient: env.user1.id().clone(),
        })
    );
    assert_eq!(env.ft1.ft_balance_of(env.user1.id()).await.unwrap(), 0);
    assert_eq!(
        env.ft1.ft_balance_of(env.swap_intent.id()).await.unwrap(),
        1000,
    );

    // try to lost_found before storage_deposit
    assert!(!env
        .user1
        .lost_found(env.swap_intent.id(), &intent_id)
        .await
        .unwrap());
    assert_eq!(
        env.swap_intent
            .get_swap_intent(&intent_id)
            .await
            .unwrap()
            .unwrap()
            .as_unlocked()
            .unwrap()
            .as_lost(),
        Some(&LostAsset {
            asset: Asset::Ft(FtAmount::new(env.ft1.id().clone(), 1000)),
            recipient: env.user1.id().clone(),
        })
    );
    assert_eq!(env.ft1.ft_balance_of(env.user1.id()).await.unwrap(), 0);
    assert_eq!(
        env.ft1.ft_balance_of(env.swap_intent.id()).await.unwrap(),
        1000,
    );

    env.user1
        .ft_storage_deposit(env.ft1.id(), None)
        .await
        .unwrap();
    assert!(env
        .user1
        .lost_found(env.swap_intent.id(), &intent_id)
        .await
        .unwrap());

    assert_eq!(
        env.swap_intent.get_swap_intent(&intent_id).await.unwrap(),
        None,
    );

    assert_eq!(env.ft1.ft_balance_of(env.user1.id()).await.unwrap(), 1000);
    assert_eq!(
        env.ft1.ft_balance_of(env.swap_intent.id()).await.unwrap(),
        0
    );
}

// TODO: test_rollback_lost_found_nft
