use std::time::Duration;

use defuse_contracts::intents::swap::{
    Asset, AssetWithAccount, CreateSwapIntentAction, Deadline, ExecuteSwapIntentAction, FtAmount,
    GenericAccount, NearAsset, SwapIntentAction,
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

    env.ft_mint(env.ft1.id(), env.user2.id(), 500)
        .await
        .unwrap();
    env.ft_mint(env.ft2.id(), env.user2.id(), 500)
        .await
        .unwrap();

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
                    asset: NearAsset::Nep141(FtAmount::new(env.ft1.id().clone(), 500))
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

    assert!(env.user1.view_account().await.unwrap().balance <= NearToken::from_near(5));

    assert!(!env
        .user2
        .swap_intent_action(
            env.swap_intent.id(),
            Asset::Near(NearAsset::Nep141(FtAmount::new(env.ft2.id().clone(), 500))),
            SwapIntentAction::Execute(ExecuteSwapIntentAction {
                id: intent_id.clone(),
                recipient: GenericAccount::Near(env.user2.id().clone()),
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

    assert_eq!(env.ft2.ft_balance_of(env.user2.id()).await.unwrap(), 500);
}
