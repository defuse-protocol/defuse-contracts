use std::time::{Duration, SystemTime};

use defuse_contracts::intents::swap::{Asset, CreateSwapIntentAction, Deadline, FtAmount, NftItem};
use near_sdk::NearToken;

use crate::{
    tests::account::AccountShardExt,
    utils::{ft::FtExt, nft::NftExt, Sandbox},
};

use super::SwapIntentShard;

#[tokio::test]
async fn test_rollback_native_intent() {
    let sandbox = Sandbox::new().await.unwrap();
    let dao = sandbox
        .create_subaccount("dao", NearToken::from_near(100))
        .await
        .unwrap();
    let swap_intent_shard = dao.deploy_swap_intent_shard("swap-intent").await;
    let user = sandbox
        .create_subaccount("user", NearToken::from_near(10))
        .await
        .unwrap();

    assert!(
        user.create_swap_intent(
            swap_intent_shard.id(),
            Asset::Native(NearToken::from_near(5)),
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
    );

    assert!(user.view_account().await.unwrap().balance < NearToken::from_near(5));

    assert!(
        user.rollback_intent(swap_intent_shard.id(), &"1".to_string())
            .await
    );

    assert_eq!(
        swap_intent_shard
            .as_account()
            .get_swap_intent(&"1".to_string())
            .await,
        None
    );
    assert!(user.view_account().await.unwrap().balance > NearToken::from_near(9));
}

#[tokio::test]
async fn test_rollback_ft_intent() {
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

    assert!(
        user.create_swap_intent(
            swap_intent_shard.id(),
            Asset::Ft(FtAmount {
                token: ft_token.id().clone(),
                amount: 1000,
            }),
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
    );

    assert_eq!(ft_token.as_account().ft_balance_of(user.id()).await, 0);
    assert_eq!(
        ft_token
            .as_account()
            .ft_balance_of(swap_intent_shard.id())
            .await,
        1000
    );

    assert!(
        user.rollback_intent(swap_intent_shard.id(), &"1".to_string())
            .await
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

#[tokio::test]
async fn test_rollback_nft_intent() {
    let sandbox = Sandbox::new().await.unwrap();
    let dao = sandbox
        .create_subaccount("dao", NearToken::from_near(100))
        .await
        .unwrap();
    let account_shard = dao.deploy_account_shard("account-shard", None).await;
    let swap_intent_shard = dao.deploy_swap_intent_shard("swap-intent").await;
    let user = sandbox
        .create_subaccount("user", NearToken::from_near(10))
        .await
        .unwrap();

    let derivation_path = "user-owned".to_string();
    user.create_account(account_shard.id(), &derivation_path, None)
        .await;

    assert!(
        user.create_swap_intent(
            swap_intent_shard.id(),
            Asset::Nft(NftItem {
                collection: account_shard.id().clone(),
                token_id: derivation_path.clone(),
            }),
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
    );

    assert_eq!(
        &account_shard
            .as_account()
            .nft_token(&derivation_path)
            .await
            .unwrap()
            .owner_id,
        swap_intent_shard.id(),
    );

    assert!(
        user.rollback_intent(swap_intent_shard.id(), &"1".to_string())
            .await
    );

    assert_eq!(
        &account_shard
            .as_account()
            .nft_token(&derivation_path)
            .await
            .unwrap()
            .owner_id,
        user.id(),
    );
}
