use std::time::{Duration, SystemTime};

use defuse_contracts::intents::swap::{
    Asset, CreateSwapIntentAction, Deadline, ExecuteSwapIntentAction, FtAmount,
};
use near_sdk::NearToken;

use crate::utils::{ft::FtExt, Sandbox};

use super::SwapIntentShard;

#[tokio::test]
async fn test_execute_wrong_asset() {
    let sandbox = Sandbox::new().await.unwrap();
    let ft_token1 = sandbox
        .root_account()
        .deploy_ft_token("ft-token-1")
        .await
        .unwrap();
    let ft_token2 = sandbox
        .root_account()
        .deploy_ft_token("ft-token-2")
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
        .ft_storage_deposit(ft_token1.id(), None)
        .await
        .unwrap();
    user2
        .ft_storage_deposit(ft_token1.id(), None)
        .await
        .unwrap();
    user2
        .ft_storage_deposit(ft_token2.id(), None)
        .await
        .unwrap();
    swap_intent_shard
        .as_account()
        .ft_storage_deposit(ft_token1.id(), None)
        .await
        .unwrap();
    swap_intent_shard
        .as_account()
        .ft_storage_deposit(ft_token2.id(), None)
        .await
        .unwrap();

    ft_token1
        .as_account()
        .ft_transfer(ft_token1.id(), user2.id(), 500, None)
        .await
        .unwrap();

    ft_token2
        .as_account()
        .ft_transfer(ft_token2.id(), user2.id(), 500, None)
        .await
        .unwrap();

    let intent_id = "1".to_string();

    assert!(user1
        .create_swap_intent(
            swap_intent_shard.id(),
            Asset::Native(NearToken::from_near(5)),
            CreateSwapIntentAction {
                id: intent_id.clone(),
                asset_out: Asset::Ft(FtAmount {
                    token: ft_token1.id().clone(),
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

    assert!(swap_intent_shard
        .as_account()
        .get_swap_intent(&intent_id)
        .await
        .unwrap()
        .unwrap()
        .as_unlocked()
        .unwrap()
        .is_available());

    assert!(user1.view_account().await.unwrap().balance <= NearToken::from_near(5));

    assert!(!user2
        .execute_swap_intent(
            swap_intent_shard.id(),
            Asset::Ft(FtAmount {
                token: ft_token2.id().clone(),
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
        .is_available());

    assert_eq!(
        ft_token2
            .as_account()
            .ft_balance_of(user2.id())
            .await
            .unwrap(),
        500
    );
}
