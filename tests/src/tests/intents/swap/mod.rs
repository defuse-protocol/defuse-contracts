use std::time::Duration;

use defuse_contracts::intents::swap::{
    Asset, CreateSwapIntentAction, ExecuteSwapIntentAction, Expiration, FtAmount, NftItem,
};

use env::Env;
use near_sdk::NearToken;

use crate::{
    tests::account::AccountShardExt,
    utils::{ft::FtExt, nft::NftExt},
};

pub use swap_intent_shard::*;

mod duplicate;
mod env;
mod expired;
mod lost_found;
mod rollback;
mod swap_intent_shard;
mod wrong_asset;
mod zero_amount;

/// Completely synthetic case, but still a valid one
#[tokio::test]
async fn test_swap_native_to_native() {
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

    assert!(env.user1.view_account().await.unwrap().balance < NearToken::from_near(7));

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
        env.swap_intent.get_swap_intent(&intent_id).await.unwrap(),
        None,
    );

    assert!(env.user1.view_account().await.unwrap().balance > NearToken::from_near(11));
    assert!(env.user2.view_account().await.unwrap().balance > NearToken::from_near(7));
}

#[tokio::test]
async fn test_swap_native_to_ft() {
    let env = Env::new().await.unwrap();
    env.root_account()
        .ft_storage_deposit_many(
            env.ft1.id(),
            &[env.user1.id(), env.user2.id(), env.swap_intent.id()],
        )
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
                asset_out: Asset::Ft(FtAmount::new(env.ft1.id().clone(), 500,)),
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

    assert!(env.user1.view_account().await.unwrap().balance <= NearToken::from_near(5));

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
        env.swap_intent.get_swap_intent(&intent_id).await.unwrap(),
        None,
    );

    assert_eq!(env.ft1.ft_balance_of(env.user1.id()).await.unwrap(), 500);
    assert_eq!(env.ft1.ft_balance_of(env.user2.id()).await.unwrap(), 0);
    assert!(env.user2.view_account().await.unwrap().balance > NearToken::from_near(14));

    assert_eq!(
        env.ft1.ft_balance_of(env.swap_intent.id()).await.unwrap(),
        0,
    );
}

#[tokio::test]
async fn test_swap_native_to_nft() {
    let env = Env::new().await.unwrap();

    let derivation_path = "user2-owned".to_string();
    env.user2
        .create_account(env.account_shard1.id(), &derivation_path, None)
        .await
        .unwrap();

    let intent_id = "1".to_string();

    assert!(env
        .user1
        .create_swap_intent(
            env.swap_intent.id(),
            Asset::Native(NearToken::from_near(5)),
            CreateSwapIntentAction {
                id: intent_id.clone(),
                asset_out: Asset::Nft(NftItem {
                    collection: env.account_shard1.id().clone(),
                    token_id: derivation_path.clone(),
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

    assert!(env.user1.view_account().await.unwrap().balance <= NearToken::from_near(5));

    assert!(env
        .user2
        .execute_swap_intent(
            env.swap_intent.id(),
            Asset::Nft(NftItem {
                collection: env.account_shard1.id().clone(),
                token_id: derivation_path.clone(),
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

    assert!(env.user1.view_account().await.unwrap().balance <= NearToken::from_near(5));
    assert!(env.user2.view_account().await.unwrap().balance >= NearToken::from_near(14));
    assert_eq!(
        &env.account_shard1
            .nft_token(&derivation_path)
            .await
            .unwrap()
            .unwrap()
            .owner_id,
        env.user1.id(),
    );
}

#[tokio::test]
async fn test_swap_ft_to_native() {
    let env = Env::new().await.unwrap();
    env.root_account()
        .ft_storage_deposit_many(
            env.ft1.id(),
            &[env.user1.id(), env.user2.id(), env.swap_intent.id()],
        )
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

    assert!(env
        .swap_intent
        .as_account()
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
        env.swap_intent.get_swap_intent(&intent_id).await.unwrap(),
        None,
    );

    assert_eq!(env.ft1.ft_balance_of(env.user1.id()).await.unwrap(), 0);
    assert_eq!(env.ft1.ft_balance_of(env.user2.id()).await.unwrap(), 500);
    assert!(env.user1.view_account().await.unwrap().balance > NearToken::from_near(14));
    assert!(env.user2.view_account().await.unwrap().balance <= NearToken::from_near(5));

    assert_eq!(
        env.ft1.ft_balance_of(env.swap_intent.id()).await.unwrap(),
        0,
    );
}

#[tokio::test]
async fn test_swap_ft_to_ft() {
    let env = Env::new().await.unwrap();
    env.root_account()
        .ft_storage_deposit_many(
            env.ft1.id(),
            &[env.user1.id(), env.user2.id(), env.swap_intent.id()],
        )
        .await
        .unwrap();
    env.root_account()
        .ft_storage_deposit_many(
            env.ft2.id(),
            &[env.user1.id(), env.user2.id(), env.swap_intent.id()],
        )
        .await
        .unwrap();

    env.ft1.ft_mint(env.user1.id(), 1000).await.unwrap();
    env.ft2.ft_mint(env.user2.id(), 2000).await.unwrap();

    let intent_id = "1".to_string();

    assert!(env
        .user1
        .create_swap_intent(
            env.swap_intent.id(),
            Asset::Ft(FtAmount::new(env.ft1.id().clone(), 1000)),
            CreateSwapIntentAction {
                id: intent_id.clone(),
                asset_out: Asset::Ft(FtAmount::new(env.ft2.id().clone(), 2000)),
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

    assert!(env
        .user2
        .execute_swap_intent(
            env.swap_intent.id(),
            Asset::Ft(FtAmount::new(env.ft2.id().clone(), 2000)),
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
async fn test_swap_ft_to_nft() {
    let env = Env::new().await.unwrap();
    env.root_account()
        .ft_storage_deposit_many(
            env.ft1.id(),
            &[env.user1.id(), env.user2.id(), env.swap_intent.id()],
        )
        .await
        .unwrap();

    env.ft1.ft_mint(env.user1.id(), 1000).await.unwrap();

    let derivation_path = "user2-owned".to_string();
    env.user2
        .create_account(env.account_shard1.id(), &derivation_path, None)
        .await
        .unwrap();

    let intent_id = "1".to_string();

    assert!(env
        .user1
        .create_swap_intent(
            env.swap_intent.id(),
            Asset::Ft(FtAmount::new(env.ft1.id().clone(), 1000)),
            CreateSwapIntentAction {
                id: intent_id.clone(),
                asset_out: Asset::Nft(NftItem {
                    collection: env.account_shard1.id().clone(),
                    token_id: derivation_path.clone(),
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

    assert!(env
        .user2
        .execute_swap_intent(
            env.swap_intent.id(),
            Asset::Nft(NftItem {
                collection: env.account_shard1.id().clone(),
                token_id: derivation_path.clone(),
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
    assert_eq!(env.ft1.ft_balance_of(env.user2.id()).await.unwrap(), 1000);
    assert_eq!(
        env.ft1
            .ft_balance_of(env.account_shard1.id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        &env.account_shard1
            .nft_token(&derivation_path)
            .await
            .unwrap()
            .unwrap()
            .owner_id,
        env.user1.id(),
    );
}

#[tokio::test]
async fn test_swap_nft_to_native() {
    let env = Env::new().await.unwrap();

    let derivation_path = "user1-owned".to_string();
    env.user1
        .create_account(env.account_shard1.id(), &derivation_path, None)
        .await
        .unwrap();

    let intent_id = "1".to_string();

    assert!(env
        .user1
        .create_swap_intent(
            env.swap_intent.id(),
            Asset::Nft(NftItem {
                collection: env.account_shard1.id().clone(),
                token_id: derivation_path.clone(),
            }),
            CreateSwapIntentAction {
                id: intent_id.clone(),
                asset_out: Asset::Native(NearToken::from_near(5)),
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

    assert_eq!(
        &env.account_shard1
            .nft_token(&derivation_path)
            .await
            .unwrap()
            .unwrap()
            .owner_id,
        env.swap_intent.id(),
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
        env.swap_intent.get_swap_intent(&intent_id).await.unwrap(),
        None,
    );

    assert_eq!(
        &env.account_shard1
            .nft_token(&derivation_path)
            .await
            .unwrap()
            .unwrap()
            .owner_id,
        env.user2.id(),
    );
    assert!(env.user2.view_account().await.unwrap().balance <= NearToken::from_near(5));
}

#[tokio::test]
async fn test_swap_nft_to_nft() {
    let env = Env::new().await.unwrap();

    let derivation_path_1 = "user1-owned".to_string();
    env.user1
        .create_account(env.account_shard1.id(), &derivation_path_1, None)
        .await
        .unwrap();
    let derivation_path_2 = "user2-owned".to_string();
    env.user2
        .create_account(env.account_shard1.id(), &derivation_path_2, None)
        .await
        .unwrap();

    let intent_id = "1".to_string();

    assert!(env
        .user1
        .create_swap_intent(
            env.swap_intent.id(),
            Asset::Nft(NftItem {
                collection: env.account_shard1.id().clone(),
                token_id: derivation_path_1.clone(),
            }),
            CreateSwapIntentAction {
                id: intent_id.clone(),
                asset_out: Asset::Nft(NftItem {
                    collection: env.account_shard1.id().clone(),
                    token_id: derivation_path_2.clone(),
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

    assert_eq!(
        &env.account_shard1
            .nft_token(&derivation_path_1)
            .await
            .unwrap()
            .unwrap()
            .owner_id,
        env.swap_intent.id(),
    );

    assert!(env
        .user2
        .execute_swap_intent(
            env.swap_intent.id(),
            Asset::Nft(NftItem {
                collection: env.account_shard1.id().clone(),
                token_id: derivation_path_2.clone(),
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

    assert_eq!(
        &env.account_shard1
            .nft_token(&derivation_path_1)
            .await
            .unwrap()
            .unwrap()
            .owner_id,
        env.user2.id(),
    );
    assert_eq!(
        &env.account_shard1
            .nft_token(&derivation_path_2)
            .await
            .unwrap()
            .unwrap()
            .owner_id,
        env.user1.id(),
    );
}
