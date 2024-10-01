use std::time::Duration;

use defuse_contracts::{
    intents::swap::{
        Asset, AssetWithAccount, CreateSwapIntentAction, FtAmount, NearAsset, NftItem,
        SwapIntentAction, SwapIntentStatus,
    },
    utils::Deadline,
};
use near_sdk::NearToken;

use crate::{
    tests::account::AccountShardExt,
    utils::{ft::FtExt, nft::NftExt},
};

use super::{Env, SwapIntentShard};

#[tokio::test]
async fn test_rollback_native_intent() {
    let env = Env::new().await.unwrap();

    assert!(env
        .user1
        .swap_intent_action(
            env.swap_intent.id(),
            Asset::Near(NearAsset::Native {
                amount: NearToken::from_near(5),
            }),
            SwapIntentAction::Create(CreateSwapIntentAction {
                id: "1".to_string(),
                asset_out: AssetWithAccount::Near {
                    account: env.user1.id().clone(),
                    asset: NearAsset::Native {
                        amount: NearToken::from_near(1)
                    },
                },
                lockup_until: None,
                expiration: Deadline::timeout(Duration::from_secs(60)),
                referral: None,
            }),
        )
        .await
        .unwrap());

    assert!(env.user1.view_account().await.unwrap().balance < NearToken::from_near(5));

    assert!(env
        .user1
        .rollback_intent(env.swap_intent.id(), &"1".to_string())
        .await
        .unwrap());

    assert_eq!(
        env.swap_intent
            .get_intent(&"1".to_string())
            .await
            .unwrap()
            .unwrap()
            .as_unlocked()
            .unwrap()
            .status,
        SwapIntentStatus::RolledBack,
    );
    assert!(env.user1.view_account().await.unwrap().balance > NearToken::from_near(9));
}

#[tokio::test]
async fn test_rollback_ft_intent() {
    let env = Env::new().await.unwrap();
    env.ft_storage_deposit(env.ft1.id(), &[env.user1.id(), env.swap_intent.id()])
        .await
        .unwrap();
    env.ft_mint(env.ft1.id(), env.user1.id(), 1000)
        .await
        .unwrap();

    assert!(env
        .user1
        .swap_intent_action(
            env.swap_intent.id(),
            Asset::Near(NearAsset::Nep141(FtAmount::new(env.ft1.id().clone(), 1000))),
            SwapIntentAction::Create(CreateSwapIntentAction {
                id: "1".to_string(),
                asset_out: AssetWithAccount::Near {
                    account: env.user1.id().clone(),
                    asset: NearAsset::Native {
                        amount: NearToken::from_near(5),
                    },
                },
                lockup_until: None,
                expiration: Deadline::timeout(Duration::from_secs(60)),
                referral: None,
            }),
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
        .rollback_intent(env.swap_intent.id(), &"1".to_string())
        .await
        .unwrap());
    assert_eq!(
        env.swap_intent
            .get_intent(&"1".to_string())
            .await
            .unwrap()
            .unwrap()
            .as_unlocked()
            .unwrap()
            .status,
        SwapIntentStatus::RolledBack,
    );

    assert_eq!(env.ft1.ft_balance_of(env.user1.id()).await.unwrap(), 1000);
    assert_eq!(
        env.ft1.ft_balance_of(env.swap_intent.id()).await.unwrap(),
        0
    );
}

#[tokio::test]
async fn test_rollback_nft_intent() {
    let env = Env::new().await.unwrap();

    let derivation_path = "user-owned".to_string();
    env.user1
        .create_account(env.account_shard1.id(), &derivation_path, None)
        .await
        .unwrap();

    assert!(env
        .user1
        .swap_intent_action(
            env.swap_intent.id(),
            Asset::Near(NearAsset::Nep171(NftItem {
                collection: env.account_shard1.id().clone(),
                token_id: derivation_path.clone(),
            })),
            SwapIntentAction::Create(CreateSwapIntentAction {
                id: "1".to_string(),
                asset_out: AssetWithAccount::Near {
                    account: env.user1.id().clone(),
                    asset: NearAsset::Native {
                        amount: NearToken::from_near(5),
                    },
                },
                lockup_until: None,
                expiration: Deadline::timeout(Duration::from_secs(60)),
                referral: None,
            }),
        )
        .await
        .unwrap());

    assert_eq!(
        &env.account_shard1
            .self_nft_token(&derivation_path)
            .await
            .unwrap()
            .unwrap()
            .owner_id,
        env.swap_intent.id(),
    );

    assert!(env
        .user1
        .rollback_intent(env.swap_intent.id(), &"1".to_string())
        .await
        .unwrap());

    assert_eq!(
        &env.account_shard1
            .self_nft_token(&derivation_path)
            .await
            .unwrap()
            .unwrap()
            .owner_id,
        env.user1.id(),
    );
}
