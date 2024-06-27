use std::time::{Duration, SystemTime};

use defuse_contracts::intents::swap::{
    Asset, CreateSwapIntentAction, Deadline, ExecuteSwapIntentAction, FtAmount, NftItem,
};

use near_sdk::NearToken;

use crate::{
    tests::account::AccountShardExt,
    utils::{ft::FtExt, nft::NftExt, Sandbox},
};

pub use swap_intent_shard::*;

mod duplicate;
mod lost_found;
mod rollback;
mod swap_intent_shard;

/// Completely synthetic case, but still a valid one
#[tokio::test]
async fn test_swap_native_to_native() {
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

    assert!(user1
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

    assert!(user1.view_account().await.unwrap().balance < NearToken::from_near(7));

    assert!(user2
        .fulfill_swap_intent(
            swap_intent_shard.id(),
            Asset::Native(NearToken::from_near(5)),
            ExecuteSwapIntentAction {
                id: "1".to_string(),
                recipient: None,
            },
        )
        .await
        .unwrap());

    assert!(user1.view_account().await.unwrap().balance > NearToken::from_near(11));
    assert!(user2.view_account().await.unwrap().balance > NearToken::from_near(17));
}

#[tokio::test]
async fn test_swap_native_to_ft() {
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
    user2.ft_storage_deposit(ft_token.id(), None).await.unwrap();
    swap_intent_shard
        .as_account()
        .ft_storage_deposit(ft_token.id(), None)
        .await
        .unwrap();

    ft_token
        .as_account()
        .ft_transfer(ft_token.id(), user2.id(), 500, None)
        .await
        .unwrap();

    assert_eq!(
        ft_token
            .as_account()
            .ft_balance_of(user1.id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        ft_token
            .as_account()
            .ft_balance_of(user2.id())
            .await
            .unwrap(),
        500
    );

    assert!(user1
        .create_swap_intent(
            swap_intent_shard.id(),
            Asset::Native(NearToken::from_near(5)),
            CreateSwapIntentAction {
                id: "1".to_string(),
                asset_out: Asset::Ft(FtAmount {
                    token: ft_token.id().clone(),
                    amount: 500,
                }),
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

    assert!(user1.view_account().await.unwrap().balance <= NearToken::from_near(5));

    assert!(user2
        .fulfill_swap_intent(
            swap_intent_shard.id(),
            Asset::Ft(FtAmount {
                token: ft_token.id().clone(),
                amount: 500,
            }),
            ExecuteSwapIntentAction {
                id: "1".to_string(),
                recipient: None,
            },
        )
        .await
        .unwrap());

    assert_eq!(
        ft_token
            .as_account()
            .ft_balance_of(user1.id())
            .await
            .unwrap(),
        500
    );
    assert_eq!(
        ft_token
            .as_account()
            .ft_balance_of(user2.id())
            .await
            .unwrap(),
        0
    );
    assert!(user2.view_account().await.unwrap().balance > NearToken::from_near(14));

    assert_eq!(
        ft_token
            .as_account()
            .ft_balance_of(swap_intent_shard.id())
            .await
            .unwrap(),
        0,
    );
}

#[tokio::test]
async fn test_swap_native_to_ft_no_deposit() {
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

    user2.ft_storage_deposit(ft_token.id(), None).await.unwrap();
    swap_intent_shard
        .as_account()
        .ft_storage_deposit(ft_token.id(), None)
        .await
        .unwrap();

    ft_token
        .as_account()
        .ft_transfer(ft_token.id(), user2.id(), 500, None)
        .await
        .unwrap();

    assert_eq!(
        ft_token
            .as_account()
            .ft_balance_of(user1.id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        ft_token
            .as_account()
            .ft_balance_of(user2.id())
            .await
            .unwrap(),
        500
    );

    let intent_id = "1".to_string();
    assert!(user1
        .create_swap_intent(
            swap_intent_shard.id(),
            Asset::Native(NearToken::from_near(5)),
            CreateSwapIntentAction {
                id: intent_id.clone(),
                asset_out: Asset::Ft(FtAmount {
                    token: ft_token.id().clone(),
                    amount: 500,
                }),
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

    assert!(user1.view_account().await.unwrap().balance <= NearToken::from_near(5));

    assert!(!user2
        .fulfill_swap_intent(
            swap_intent_shard.id(),
            Asset::Ft(FtAmount {
                token: ft_token.id().clone(),
                amount: 500,
            }),
            ExecuteSwapIntentAction {
                id: intent_id.clone(),
                recipient: None,
            },
        )
        .await
        .unwrap());

    assert!(swap_intent_shard
        .as_account()
        .get_swap_intent(&intent_id)
        .await
        .unwrap()
        .unwrap()
        .as_unlocked()
        .unwrap()
        .as_available()
        .is_some());

    assert_eq!(
        ft_token
            .as_account()
            .ft_balance_of(user2.id())
            .await
            .unwrap(),
        500
    );
    assert_eq!(
        ft_token
            .as_account()
            .ft_balance_of(swap_intent_shard.id())
            .await
            .unwrap(),
        0,
    );
    assert_eq!(
        ft_token
            .as_account()
            .ft_balance_of(user1.id())
            .await
            .unwrap(),
        0
    );

    assert!(user2.view_account().await.unwrap().balance < NearToken::from_near(10));
}

#[tokio::test]
async fn test_swap_native_to_nft() {
    let sandbox = Sandbox::new().await.unwrap();
    let dao = sandbox
        .create_subaccount("dao", NearToken::from_near(100))
        .await
        .unwrap();
    let account_shard = dao
        .deploy_account_shard("account-shard", None)
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

    let derivation_path = "user2-owned".to_string();
    user2
        .create_account(account_shard.id(), &derivation_path, None)
        .await
        .unwrap();

    assert!(user1
        .create_swap_intent(
            swap_intent_shard.id(),
            Asset::Native(NearToken::from_near(5)),
            CreateSwapIntentAction {
                id: "1".to_string(),
                asset_out: Asset::Nft(NftItem {
                    collection: account_shard.id().clone(),
                    token_id: derivation_path.clone(),
                }),
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

    assert!(user1.view_account().await.unwrap().balance <= NearToken::from_near(5));

    assert!(user2
        .fulfill_swap_intent(
            swap_intent_shard.id(),
            Asset::Nft(NftItem {
                collection: account_shard.id().clone(),
                token_id: derivation_path.clone(),
            }),
            ExecuteSwapIntentAction {
                id: "1".to_string(),
                recipient: None,
            },
        )
        .await
        .unwrap());

    assert!(user1.view_account().await.unwrap().balance <= NearToken::from_near(5));
    assert!(user2.view_account().await.unwrap().balance >= NearToken::from_near(14));
    assert_eq!(
        &account_shard
            .as_account()
            .nft_token(&derivation_path)
            .await
            .unwrap()
            .unwrap()
            .owner_id,
        user1.id(),
    );
}

#[tokio::test]
async fn test_swap_ft_to_native() {
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
    user2.ft_storage_deposit(ft_token.id(), None).await.unwrap();
    swap_intent_shard
        .as_account()
        .ft_storage_deposit(ft_token.id(), None)
        .await
        .unwrap();

    ft_token
        .as_account()
        .ft_transfer(ft_token.id(), user1.id(), 500, None)
        .await
        .unwrap();

    assert_eq!(
        ft_token
            .as_account()
            .ft_balance_of(user1.id())
            .await
            .unwrap(),
        500
    );
    assert_eq!(
        ft_token
            .as_account()
            .ft_balance_of(user2.id())
            .await
            .unwrap(),
        0
    );

    assert!(user1
        .create_swap_intent(
            swap_intent_shard.id(),
            Asset::Ft(FtAmount {
                token: ft_token.id().clone(),
                amount: 500,
            }),
            CreateSwapIntentAction {
                id: "1".to_string(),
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

    assert_eq!(
        ft_token
            .as_account()
            .ft_balance_of(user1.id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        ft_token
            .as_account()
            .ft_balance_of(swap_intent_shard.id())
            .await
            .unwrap(),
        500
    );

    assert!(user2
        .fulfill_swap_intent(
            swap_intent_shard.id(),
            Asset::Native(NearToken::from_near(5)),
            ExecuteSwapIntentAction {
                id: "1".to_string(),
                recipient: None,
            },
        )
        .await
        .unwrap());

    assert_eq!(
        ft_token
            .as_account()
            .ft_balance_of(user1.id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        ft_token
            .as_account()
            .ft_balance_of(user2.id())
            .await
            .unwrap(),
        500
    );
    assert!(user1.view_account().await.unwrap().balance > NearToken::from_near(14));
    assert!(user2.view_account().await.unwrap().balance <= NearToken::from_near(5));

    assert_eq!(
        ft_token
            .as_account()
            .ft_balance_of(swap_intent_shard.id())
            .await
            .unwrap(),
        0,
    );
}

#[tokio::test]
async fn test_swap_ft_to_ft() {
    let sandbox = Sandbox::new().await.unwrap();
    let ft_token_a = sandbox
        .root_account()
        .deploy_ft_token("ft-token-a")
        .await
        .unwrap();
    let ft_token_b = sandbox
        .root_account()
        .deploy_ft_token("ft-token-b")
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

    user1
        .ft_storage_deposit(ft_token_a.id(), None)
        .await
        .unwrap();
    user1
        .ft_storage_deposit(ft_token_b.id(), None)
        .await
        .unwrap();
    user2
        .ft_storage_deposit(ft_token_a.id(), None)
        .await
        .unwrap();
    user2
        .ft_storage_deposit(ft_token_b.id(), None)
        .await
        .unwrap();
    swap_intent_shard
        .as_account()
        .ft_storage_deposit(ft_token_a.id(), None)
        .await
        .unwrap();
    swap_intent_shard
        .as_account()
        .ft_storage_deposit(ft_token_b.id(), None)
        .await
        .unwrap();

    ft_token_a
        .as_account()
        .ft_transfer(ft_token_a.id(), user1.id(), 1000, None)
        .await
        .unwrap();
    ft_token_b
        .as_account()
        .ft_transfer(ft_token_b.id(), user2.id(), 2000, None)
        .await
        .unwrap();

    assert_eq!(
        ft_token_a
            .as_account()
            .ft_balance_of(user1.id())
            .await
            .unwrap(),
        1000
    );
    assert_eq!(
        ft_token_b
            .as_account()
            .ft_balance_of(user1.id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        ft_token_a
            .as_account()
            .ft_balance_of(user2.id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        ft_token_b
            .as_account()
            .ft_balance_of(user2.id())
            .await
            .unwrap(),
        2000
    );

    assert!(user1
        .create_swap_intent(
            swap_intent_shard.id(),
            Asset::Ft(FtAmount {
                token: ft_token_a.id().clone(),
                amount: 1000,
            }),
            CreateSwapIntentAction {
                id: "1".to_string(),
                asset_out: Asset::Ft(FtAmount {
                    token: ft_token_b.id().clone(),
                    amount: 2000,
                }),
                recipient: None,
                // TODO
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

    assert_eq!(
        ft_token_a
            .as_account()
            .ft_balance_of(user1.id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        ft_token_a
            .as_account()
            .ft_balance_of(swap_intent_shard.id())
            .await
            .unwrap(),
        1000
    );

    assert!(user2
        .fulfill_swap_intent(
            swap_intent_shard.id(),
            Asset::Ft(FtAmount {
                token: ft_token_b.id().clone(),
                amount: 2000,
            }),
            ExecuteSwapIntentAction {
                id: "1".to_string(),
                recipient: None,
            },
        )
        .await
        .unwrap());

    assert_eq!(
        ft_token_a
            .as_account()
            .ft_balance_of(user1.id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        ft_token_b
            .as_account()
            .ft_balance_of(user1.id())
            .await
            .unwrap(),
        2000
    );
    assert_eq!(
        ft_token_a
            .as_account()
            .ft_balance_of(user2.id())
            .await
            .unwrap(),
        1000
    );
    assert_eq!(
        ft_token_b
            .as_account()
            .ft_balance_of(user2.id())
            .await
            .unwrap(),
        0
    );

    assert_eq!(
        ft_token_a
            .as_account()
            .ft_balance_of(swap_intent_shard.id())
            .await
            .unwrap(),
        0,
    );
    assert_eq!(
        ft_token_b
            .as_account()
            .ft_balance_of(swap_intent_shard.id())
            .await
            .unwrap(),
        0,
    );
}

#[tokio::test]
async fn test_swap_ft_to_nft() {
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
    let account_shard = dao
        .deploy_account_shard("account-shard", None)
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
    user2.ft_storage_deposit(ft_token.id(), None).await.unwrap();
    swap_intent_shard
        .as_account()
        .ft_storage_deposit(ft_token.id(), None)
        .await
        .unwrap();

    ft_token
        .as_account()
        .ft_transfer(ft_token.id(), user1.id(), 1000, None)
        .await
        .unwrap();
    assert_eq!(
        ft_token
            .as_account()
            .ft_balance_of(user1.id())
            .await
            .unwrap(),
        1000
    );
    assert_eq!(
        ft_token
            .as_account()
            .ft_balance_of(user2.id())
            .await
            .unwrap(),
        0
    );

    let derivation_path = "user2-owned".to_string();
    user2
        .create_account(account_shard.id(), &derivation_path, None)
        .await
        .unwrap();

    assert!(user1
        .create_swap_intent(
            swap_intent_shard.id(),
            Asset::Ft(FtAmount {
                token: ft_token.id().clone(),
                amount: 1000,
            }),
            CreateSwapIntentAction {
                id: "1".to_string(),
                asset_out: Asset::Nft(NftItem {
                    collection: account_shard.id().clone(),
                    token_id: derivation_path.clone(),
                }),
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

    assert_eq!(
        ft_token
            .as_account()
            .ft_balance_of(user1.id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        ft_token
            .as_account()
            .ft_balance_of(swap_intent_shard.id())
            .await
            .unwrap(),
        1000
    );

    assert!(user2
        .fulfill_swap_intent(
            swap_intent_shard.id(),
            Asset::Nft(NftItem {
                collection: account_shard.id().clone(),
                token_id: derivation_path.clone(),
            }),
            ExecuteSwapIntentAction {
                id: "1".to_string(),
                recipient: None,
            },
        )
        .await
        .unwrap());

    assert_eq!(
        ft_token
            .as_account()
            .ft_balance_of(user1.id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        ft_token
            .as_account()
            .ft_balance_of(user2.id())
            .await
            .unwrap(),
        1000
    );
    assert_eq!(
        ft_token
            .as_account()
            .ft_balance_of(account_shard.id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        &account_shard
            .as_account()
            .nft_token(&derivation_path)
            .await
            .unwrap()
            .unwrap()
            .owner_id,
        user1.id(),
    );
}

#[tokio::test]
async fn test_swap_nft_to_native() {
    let sandbox = Sandbox::new().await.unwrap();
    let dao = sandbox
        .create_subaccount("dao", NearToken::from_near(100))
        .await
        .unwrap();
    let account_shard = dao
        .deploy_account_shard("account-shard", None)
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

    let derivation_path = "user1-owned".to_string();
    user1
        .create_account(account_shard.id(), &derivation_path, None)
        .await
        .unwrap();

    assert!(user1
        .create_swap_intent(
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
                    (SystemTime::now() + Duration::from_secs(60))
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                ),
            },
        )
        .await
        .unwrap());

    assert_eq!(
        &account_shard
            .as_account()
            .nft_token(&derivation_path)
            .await
            .unwrap()
            .unwrap()
            .owner_id,
        swap_intent_shard.id(),
    );

    assert!(user2
        .fulfill_swap_intent(
            swap_intent_shard.id(),
            Asset::Native(NearToken::from_near(5)),
            ExecuteSwapIntentAction {
                id: "1".to_string(),
                recipient: None,
            },
        )
        .await
        .unwrap());

    assert_eq!(
        &account_shard
            .as_account()
            .nft_token(&derivation_path)
            .await
            .unwrap()
            .unwrap()
            .owner_id,
        user2.id(),
    );
    assert!(user2.view_account().await.unwrap().balance <= NearToken::from_near(5));
}

#[tokio::test]
async fn test_swap_nft_to_nft() {
    let sandbox = Sandbox::new().await.unwrap();
    let dao = sandbox
        .create_subaccount("dao", NearToken::from_near(100))
        .await
        .unwrap();
    let account_shard = dao
        .deploy_account_shard("account-shard", None)
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

    let derivation_path_1 = "user1-owned".to_string();
    user1
        .create_account(account_shard.id(), &derivation_path_1, None)
        .await
        .unwrap();
    let derivation_path_2 = "user2-owned".to_string();
    user2
        .create_account(account_shard.id(), &derivation_path_2, None)
        .await
        .unwrap();

    assert!(user1
        .create_swap_intent(
            swap_intent_shard.id(),
            Asset::Nft(NftItem {
                collection: account_shard.id().clone(),
                token_id: derivation_path_1.clone(),
            }),
            CreateSwapIntentAction {
                id: "1".to_string(),
                asset_out: Asset::Nft(NftItem {
                    collection: account_shard.id().clone(),
                    token_id: derivation_path_2.clone(),
                }),
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

    assert_eq!(
        &account_shard
            .as_account()
            .nft_token(&derivation_path_1)
            .await
            .unwrap()
            .unwrap()
            .owner_id,
        swap_intent_shard.id(),
    );

    assert!(user2
        .fulfill_swap_intent(
            swap_intent_shard.id(),
            Asset::Nft(NftItem {
                collection: account_shard.id().clone(),
                token_id: derivation_path_2.clone(),
            }),
            ExecuteSwapIntentAction {
                id: "1".to_string(),
                recipient: None,
            },
        )
        .await
        .unwrap());

    assert_eq!(
        &account_shard
            .as_account()
            .nft_token(&derivation_path_1)
            .await
            .unwrap()
            .unwrap()
            .owner_id,
        user2.id(),
    );
    assert_eq!(
        &account_shard
            .as_account()
            .nft_token(&derivation_path_2)
            .await
            .unwrap()
            .unwrap()
            .owner_id,
        user1.id(),
    );
}
