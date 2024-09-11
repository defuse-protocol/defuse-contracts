use std::time::Duration;

use defuse_contracts::{
    intents::swap::{
        Asset, AssetWithAccount, CreateSwapIntentAction, ExecuteSwapIntentAction, GenericAccount,
        NearAsset, SwapIntentAction,
    },
    utils::Deadline,
};
use near_sdk::NearToken;

use super::{Env, SwapIntentShard};

#[tokio::test]
async fn test_duplicate_native() {
    let env = Env::new().await.unwrap();
    let intent_id = "1".to_string();

    assert!(env
        .user1
        .swap_intent_action(
            env.swap_intent.id(),
            Asset::Near(NearAsset::Native {
                amount: NearToken::from_near(3)
            }),
            SwapIntentAction::Create(CreateSwapIntentAction {
                id: intent_id.clone(),
                asset_out: AssetWithAccount::Near {
                    account: env.user1.id().clone(),
                    asset: NearAsset::Native {
                        amount: NearToken::from_near(5)
                    },
                },
                lockup_until: None,
                expiration: Deadline::timeout(Duration::from_secs(60)),
                referral: None,
            }),
        )
        .await
        .unwrap());

    // cannot create intent with same ID
    assert!(env
        .user1
        .swap_intent_action(
            env.swap_intent.id(),
            Asset::Near(NearAsset::Native {
                amount: NearToken::from_near(3)
            }),
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
        .is_err());

    assert!(env
        .user2
        .swap_intent_action(
            env.swap_intent.id(),
            Asset::Near(NearAsset::Native {
                amount: NearToken::from_near(5)
            }),
            SwapIntentAction::Execute(ExecuteSwapIntentAction {
                id: intent_id.clone(),
                recipient: GenericAccount::Near(env.user2.id().clone()),
                proof: None,
            }),
        )
        .await
        .unwrap());

    // cannot execute same intent twice
    assert!(env
        .user2
        .swap_intent_action(
            env.swap_intent.id(),
            Asset::Near(NearAsset::Native {
                amount: NearToken::from_near(5)
            }),
            SwapIntentAction::Execute(ExecuteSwapIntentAction {
                id: intent_id.clone(),
                recipient: GenericAccount::Near(env.user2.id().clone()),
                proof: None,
            }),
        )
        .await
        .is_err());
}
