use std::time::Duration;

use defuse_contracts::{
    intents::swap::{Asset, AssetWithAccount, CreateSwapIntentAction, NearAsset, SwapIntentAction},
    utils::Deadline,
};
use near_sdk::NearToken;

use super::{Env, SwapIntentShard};

#[tokio::test]
async fn test_create_zero_amount() {
    let env = Env::new().await.unwrap();
    let intent_id = "1".to_string();

    assert!(env
        .user1
        .swap_intent_action(
            env.swap_intent.id(),
            Asset::Near(NearAsset::Native {
                amount: NearToken::from_near(0),
            }),
            SwapIntentAction::Create(CreateSwapIntentAction {
                id: intent_id.clone(),
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
        .is_err());

    assert_eq!(env.swap_intent.get_intent(&intent_id).await.unwrap(), None);

    assert!(env
        .user1
        .swap_intent_action(
            env.swap_intent.id(),
            Asset::Near(NearAsset::Native {
                amount: NearToken::from_near(1)
            }),
            SwapIntentAction::Create(CreateSwapIntentAction {
                id: intent_id.clone(),
                asset_out: AssetWithAccount::Near {
                    account: env.user1.id().clone(),
                    asset: NearAsset::Native {
                        amount: NearToken::from_near(0),
                    },
                },
                lockup_until: None,
                expiration: Deadline::timeout(Duration::from_secs(60)),
                referral: None,
            }),
        )
        .await
        .is_err());

    assert_eq!(env.swap_intent.get_intent(&intent_id).await.unwrap(), None);

    assert!(env.user1.view_account().await.unwrap().balance > NearToken::from_near(9));
}
