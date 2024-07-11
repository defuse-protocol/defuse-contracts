use std::time::Duration;

use defuse_contracts::intents::swap::{
    Asset, CreateSwapIntentAction, ExecuteSwapIntentAction, Expiration, FtAmount,
};
use near_sdk::NearToken;

use crate::utils::ft::FtExt;

use super::{Env, SwapIntentShard};

#[tokio::test]
async fn test_execute_wrong_asset() {
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

    env.ft1.ft_mint(env.user2.id(), 500).await.unwrap();
    env.ft2.ft_mint(env.user2.id(), 500).await.unwrap();

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
        .swap_intent
        .get_swap_intent(&intent_id)
        .await
        .unwrap()
        .unwrap()
        .as_unlocked()
        .unwrap()
        .is_available());

    assert!(env.user1.view_account().await.unwrap().balance <= NearToken::from_near(5));

    assert!(!env
        .user2
        .execute_swap_intent(
            env.swap_intent.id(),
            Asset::Ft(FtAmount::new(env.ft2.id().clone(), 500)),
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

    assert_eq!(env.ft2.ft_balance_of(env.user2.id()).await.unwrap(), 500);
}
