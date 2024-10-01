use defuse_contracts::{
    intents::swap::{
        Asset, AssetWithAccount, CreateSwapIntentAction, ExecuteSwapIntentAction, GenericAccount,
        NearAsset, SwapIntentAction, SwapIntentStatus,
    },
    utils::Deadline,
};
use near_sdk::NearToken;

use super::{Env, SwapIntentShard};

#[tokio::test]
async fn test_execute_expired() {
    let env = Env::new().await.unwrap();

    let intent_id = "1".to_string();
    let current_height = env.block_height().await;

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
                expiration: Deadline::BlockNumber(current_height + 100),
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

    assert!(env.user1.view_account().await.unwrap().balance < NearToken::from_near(7));

    env.skip_blocks(100).await;

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

    assert!(env
        .swap_intent
        .get_intent(&intent_id)
        .await
        .unwrap()
        .unwrap()
        .as_unlocked()
        .unwrap()
        .is_available());

    assert!(env
        .user1
        .rollback_intent(env.swap_intent.id(), &intent_id)
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
async fn test_rollback_locked_up() {
    let env = Env::new().await.unwrap();

    let intent_id = "1".to_string();
    let current_height = env.block_height().await;

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
                lockup_until: Some(Deadline::BlockNumber(current_height + 50)),
                expiration: Deadline::BlockNumber(current_height + 100),
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

    assert!(env
        .user1
        .rollback_intent(env.swap_intent.id(), &intent_id)
        .await
        .is_err());

    assert!(env.user1.view_account().await.unwrap().balance < NearToken::from_near(7));

    env.skip_blocks(50).await;

    assert!(env
        .user1
        .rollback_intent(env.swap_intent.id(), &intent_id)
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
