use std::time::Duration;

use defuse_contracts::{
    intents::swap::{
        Asset, AssetWithAccount, CreateSwapIntentAction, ExecuteSwapIntentAction, FtAmount,
        LostAsset, NearAsset, SwapIntentAction, SwapIntentStatus,
    },
    utils::Deadline,
};
use near_sdk::NearToken;

use crate::utils::{ft::FtExt, storage_management::StorageManagementExt};

use super::{Env, SwapIntentShard};

#[tokio::test]
async fn test_execute_assets_in_out_ft_no_deposits() {
    let env = Env::new().await.unwrap();
    env.ft_storage_deposit(&env.ft1, &[env.user1.id(), env.swap_intent.id()])
        .await
        .unwrap();
    env.ft_storage_deposit(&env.ft2, &[env.user2.id(), env.swap_intent.id()])
        .await
        .unwrap();

    env.ft_mint(&env.ft1, env.user1.id(), 1000).await.unwrap();
    env.ft_mint(&env.ft2, env.user2.id(), 2000).await.unwrap();

    let intent_id = "1".to_string();

    assert!(env
        .user1
        .swap_intent_action(
            env.swap_intent.id(),
            Asset::Near(NearAsset::Nep141(FtAmount {
                token: env.ft1.clone(),
                amount: 1000.into(),
            })),
            SwapIntentAction::Create(CreateSwapIntentAction {
                id: intent_id.clone(),
                asset_out: AssetWithAccount::Near {
                    account: env.user1.id().clone(),
                    asset: NearAsset::Nep141(FtAmount {
                        token: env.ft2.clone(),
                        amount: 2000.into(),
                    })
                },
                lockup_until: None,
                expiration: Deadline::timeout(Duration::from_secs(60)),
                referral: None,
            }),
        )
        .await
        .unwrap());

    assert!(env
        .swap_intent
        .get_intent(&intent_id)
        .await
        .unwrap()
        .unwrap()
        .as_unlocked()
        .unwrap()
        .is_available());

    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.user1.id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.swap_intent.id())
            .await
            .unwrap(),
        1000
    );

    assert!(!env
        .user2
        .swap_intent_action(
            env.swap_intent.id(),
            Asset::Near(NearAsset::Nep141(FtAmount {
                token: env.ft2.clone(),
                amount: 2000.into(),
            })),
            SwapIntentAction::Execute(ExecuteSwapIntentAction {
                id: intent_id.clone(),
                recipient: env.user2.id().clone().into(),
                proof: None,
            }),
        )
        .await
        .unwrap());

    assert!(env
        .swap_intent
        .get_intent(&intent_id)
        .await
        .unwrap()
        .unwrap()
        .as_unlocked()
        .unwrap()
        .is_available());

    env.user1.ft_storage_deposit(&env.ft2, None).await.unwrap();
    env.user2.ft_storage_deposit(&env.ft1, None).await.unwrap();

    assert!(env
        .user2
        .swap_intent_action(
            env.swap_intent.id(),
            Asset::Near(NearAsset::Nep141(FtAmount {
                token: env.ft2.clone(),
                amount: 2000.into(),
            })),
            SwapIntentAction::Execute(ExecuteSwapIntentAction {
                id: intent_id.clone(),
                recipient: env.user2.id().clone().into(),
                proof: None,
            }),
        )
        .await
        .unwrap());

    assert!(env
        .swap_intent
        .get_intent(&intent_id)
        .await
        .unwrap()
        .unwrap()
        .as_unlocked()
        .unwrap()
        .status
        .is_executed());

    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.user1.id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.ft2, env.user1.id())
            .await
            .unwrap(),
        2000
    );
    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.user2.id())
            .await
            .unwrap(),
        1000
    );
    assert_eq!(
        env.ft_token_balance_of(&env.ft2, env.user2.id())
            .await
            .unwrap(),
        0
    );

    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.swap_intent.id())
            .await
            .unwrap(),
        0,
    );
    assert_eq!(
        env.ft_token_balance_of(&env.ft2, env.swap_intent.id())
            .await
            .unwrap(),
        0,
    );
}

#[tokio::test]
async fn test_execute_asset_in_ft_no_deposit() {
    let env = Env::new().await.unwrap();
    env.ft_storage_deposit(&env.ft1, &[env.user1.id(), env.swap_intent.id()])
        .await
        .unwrap();

    env.ft_mint(&env.ft1, env.user1.id(), 500).await.unwrap();

    let intent_id = "1".to_string();
    assert!(env
        .user1
        .swap_intent_action(
            env.swap_intent.id(),
            Asset::Near(NearAsset::Nep141(FtAmount::new(env.ft1.clone(), 500))),
            SwapIntentAction::Create(CreateSwapIntentAction {
                id: intent_id.clone(),
                asset_out: AssetWithAccount::Near {
                    account: env.user1.id().clone(),
                    asset: NearAsset::Native {
                        amount: NearToken::from_near(5)
                    }
                },
                lockup_until: None,
                expiration: Deadline::timeout(Duration::from_secs(60)),
                referral: None,
            }),
        )
        .await
        .unwrap());

    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.user1.id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.swap_intent.id())
            .await
            .unwrap(),
        500
    );

    assert!(env
        .user2
        .swap_intent_action(
            env.swap_intent.id(),
            Asset::Near(NearAsset::Native {
                amount: NearToken::from_near(5)
            }),
            SwapIntentAction::Execute(ExecuteSwapIntentAction {
                id: intent_id.clone(),
                recipient: env.user2.id().clone().into(),
                proof: None,
            }),
        )
        .await
        .unwrap());

    let intent = env
        .swap_intent
        .get_intent(&intent_id)
        .await
        .unwrap()
        .unwrap();
    assert!(intent.as_unlocked().unwrap().status.is_executed());
    assert_eq!(
        intent.as_unlocked().unwrap().lost,
        Some(LostAsset::AssetIn {
            recipient: env.user2.id().clone().into(),
        }),
    );

    assert!(env.user1.view_account().await.unwrap().balance > NearToken::from_near(14));
    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.swap_intent.id())
            .await
            .unwrap(),
        500
    );
    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.user2.id())
            .await
            .unwrap(),
        0
    );
    assert!(env.user2.view_account().await.unwrap().balance <= NearToken::from_near(5));

    // no storage_deposit yet
    assert!(!env
        .user2
        .lost_found(env.swap_intent.id(), &intent_id)
        .await
        .unwrap());
    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.swap_intent.id())
            .await
            .unwrap(),
        500
    );
    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.user2.id())
            .await
            .unwrap(),
        0
    );

    env.user2.ft_storage_deposit(&env.ft1, None).await.unwrap();
    assert!(env
        .user2
        .lost_found(env.swap_intent.id(), &intent_id)
        .await
        .unwrap());

    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.swap_intent.id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.user2.id())
            .await
            .unwrap(),
        500
    );

    assert_eq!(
        env.swap_intent
            .get_intent(&intent_id)
            .await
            .unwrap()
            .unwrap()
            .as_unlocked()
            .unwrap()
            .lost,
        None,
    );
}

#[tokio::test]
async fn test_execute_asset_out_ft_no_deposit() {
    let env = Env::new().await.unwrap();
    env.ft_storage_deposit(&env.ft1, &[env.user2.id(), env.swap_intent.id()])
        .await
        .unwrap();
    env.ft_mint(&env.ft1, env.user2.id(), 500).await.unwrap();

    let intent_id = "1".to_string();
    assert!(env
        .user1
        .swap_intent_action(
            env.swap_intent.id(),
            Asset::Near(NearAsset::Native {
                amount: NearToken::from_near(5)
            }),
            SwapIntentAction::Create(CreateSwapIntentAction {
                id: intent_id.clone(),
                asset_out: AssetWithAccount::Near {
                    account: env.user1.id().clone(),
                    asset: NearAsset::Nep141(FtAmount::new(env.ft1.clone(), 500))
                },
                lockup_until: None,
                expiration: Deadline::timeout(Duration::from_secs(60)),
                referral: None,
            }),
        )
        .await
        .unwrap());

    assert!(env
        .user2
        .swap_intent_action(
            env.swap_intent.id(),
            Asset::Near(NearAsset::Nep141(FtAmount::new(env.ft1.clone(), 500))),
            SwapIntentAction::Execute(ExecuteSwapIntentAction {
                id: intent_id.clone(),
                recipient: env.user2.id().clone().into(),
                proof: None,
            }),
        )
        .await
        .unwrap());

    let intent = env
        .swap_intent
        .get_intent(&intent_id)
        .await
        .unwrap()
        .unwrap();

    assert!(intent.as_unlocked().unwrap().status.is_executed());
    assert_eq!(
        intent.as_unlocked().unwrap().lost,
        Some(LostAsset::AssetOut),
    );
    assert!(env.user1.view_account().await.unwrap().balance <= NearToken::from_near(5));
    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.swap_intent.id())
            .await
            .unwrap(),
        500
    );
    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.user1.id())
            .await
            .unwrap(),
        0
    );
    assert!(env.user2.view_account().await.unwrap().balance > NearToken::from_near(14));

    // no storage_deposit yet
    assert!(!env
        .user1
        .lost_found(env.swap_intent.id(), &intent_id)
        .await
        .unwrap());
    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.swap_intent.id())
            .await
            .unwrap(),
        500
    );
    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.user1.id())
            .await
            .unwrap(),
        0
    );

    env.user1.ft_storage_deposit(&env.ft1, None).await.unwrap();
    assert!(env
        .user1
        .lost_found(env.swap_intent.id(), &intent_id)
        .await
        .unwrap());

    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.swap_intent.id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.user1.id())
            .await
            .unwrap(),
        500
    );

    assert_eq!(
        env.swap_intent
            .get_intent(&intent_id)
            .await
            .unwrap()
            .unwrap()
            .as_unlocked()
            .unwrap()
            .lost,
        None,
    );
}

#[tokio::test]
async fn test_rollback_lost_found_ft() {
    let env = Env::new().await.unwrap();
    env.root_account()
        .ft_storage_deposit_many(&env.ft1, &[env.user1.id(), env.swap_intent.id()])
        .await
        .unwrap();

    env.ft_mint(&env.ft1, env.user1.id(), 1000).await.unwrap();

    let intent_id = "1".to_string();

    assert!(env
        .user1
        .swap_intent_action(
            env.swap_intent.id(),
            Asset::Near(NearAsset::Nep141(FtAmount::new(env.ft1.clone(), 1000))),
            SwapIntentAction::Create(CreateSwapIntentAction {
                id: intent_id.clone(),
                asset_out: AssetWithAccount::Near {
                    account: env.user1.id().clone(),
                    asset: NearAsset::Native {
                        amount: NearToken::from_near(5)
                    }
                },
                lockup_until: None,
                expiration: Deadline::timeout(Duration::from_secs(60)),
                referral: None,
            }),
        )
        .await
        .unwrap());

    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.user1.id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.swap_intent.id())
            .await
            .unwrap(),
        1000
    );

    assert!(env.user1.storage_unregister(&env.ft1, None).await.unwrap());

    assert!(!env
        .user1
        .rollback_intent(env.swap_intent.id(), &intent_id)
        .await
        .unwrap());

    let intent = env
        .swap_intent
        .get_intent(&intent_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        intent.as_unlocked().unwrap().status,
        SwapIntentStatus::RolledBack
    );
    assert_eq!(
        intent.as_unlocked().unwrap().lost,
        Some(LostAsset::AssetIn {
            recipient: env.user1.id().clone().into(),
        }),
    );

    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.user1.id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.swap_intent.id())
            .await
            .unwrap(),
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
            .get_intent(&intent_id)
            .await
            .unwrap()
            .unwrap()
            .as_unlocked()
            .unwrap()
            .lost,
        Some(LostAsset::AssetIn {
            recipient: env.user1.id().clone().into(),
        }),
    );
    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.user1.id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.swap_intent.id())
            .await
            .unwrap(),
        1000,
    );

    env.user1.ft_storage_deposit(&env.ft1, None).await.unwrap();
    assert!(env
        .user1
        .lost_found(env.swap_intent.id(), &intent_id)
        .await
        .unwrap());

    assert_eq!(
        env.swap_intent
            .get_intent(&intent_id)
            .await
            .unwrap()
            .unwrap()
            .as_unlocked()
            .unwrap()
            .lost,
        None,
    );

    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.user1.id())
            .await
            .unwrap(),
        1000
    );
    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.swap_intent.id())
            .await
            .unwrap(),
        0
    );
}
