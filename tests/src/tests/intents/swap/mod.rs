use core::time::Duration;

use defuse_contracts::intents::swap::{
    Asset, AssetWithAccount, CreateSwapIntentAction, CrossChainAsset, Deadline,
    ExecuteSwapIntentAction, FtAmount, GenericAccount, NearAsset, NftItem, SwapIntentAction,
};
use near_sdk::{json_types::U128, NearToken};
use rstest::rstest;

use crate::{
    tests::account::AccountShardExt,
    utils::{ft::FtExt, nft::NftExt},
};

use env::{Env, ACCOUNT_SHARD1, ACCOUNT_SHARD2, FT1, FT2, USER1, USER2};

pub use swap_intent_shard::*;

mod duplicate;
mod env;
mod expired;
mod lost_found;
mod rollback;
mod swap_intent_shard;
mod wrong_asset;
mod zero_amount;

#[rstest]
#[trace]
#[tokio::test]
async fn test_swap(
    #[values(
        Asset::Near(NearAsset::Native{
            amount: NearToken::from_near(3),
        }),
        Asset::Near(NearAsset::Nep141(FtAmount {
            token: FT1.clone(),
            amount: U128(500),
        })),
        Asset::Near(NearAsset::Nep171(NftItem {
            collection: ACCOUNT_SHARD1.clone(),
            token_id: "a".to_string(),
        })),
        Asset::CrossChain(CrossChainAsset {
            // for now, user1 acts as an oracle
            oracle: USER1.clone(),
            // USDT (Ethereum mainnet)
            asset: "eth:1:0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
            amount: "500".to_string(),
        })
    )]
    asset_in: Asset,
    #[values(
        Asset::Near(NearAsset::Native{
            amount: NearToken::from_near(5),
        }),
        Asset::Near(NearAsset::Nep141(FtAmount {
            token: FT2.clone(),
            amount: U128(1000),
        })),
        Asset::Near(NearAsset::Nep171(NftItem {
            collection: ACCOUNT_SHARD2.clone(),
            token_id: "b".to_string(),
        })),
        Asset::CrossChain(CrossChainAsset {
            // for now, user2 acts as an oracle
            oracle: USER2.clone(),
            // USDC (Base mainnet)
            asset: "eth:8453:0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".to_string(),
            amount: "1000".to_string(),
        })
    )]
    asset_out: Asset,
) {
    let env = Env::new().await.unwrap();
    let intent_id = "1".to_string();

    for (owner, asset) in [(env.user1.id(), &asset_in), (env.user2.id(), &asset_out)] {
        match asset {
            Asset::Near(NearAsset::Nep141(FtAmount { token, amount })) => {
                env.root_account()
                    .ft_storage_deposit_many(
                        token,
                        &[env.user1.id(), env.user2.id(), env.swap_intent.id()],
                    )
                    .await
                    .unwrap();
                env.ft_mint(token, owner, amount.0).await.unwrap();
            }
            Asset::Near(NearAsset::Nep171(NftItem {
                // must be Account Shard
                collection,
                token_id,
            })) => {
                env.root_account()
                    .create_account(collection, token_id, Some(owner.clone()))
                    .await
                    .unwrap();
            }
            _ => {}
        }
    }

    let asset_out = match asset_out {
        Asset::Near(asset) => AssetWithAccount::Near {
            account: env.user1.id().clone(),
            asset,
        },
        Asset::CrossChain(asset) => AssetWithAccount::CrossChain {
            account: "user1".to_string(),
            asset,
        },
    };

    let expiration = Deadline::timeout(Duration::from_secs(60));

    assert!(env
        .user1
        .swap_intent_action(
            env.swap_intent.id(),
            asset_in.clone(),
            SwapIntentAction::Create(CreateSwapIntentAction {
                id: intent_id.clone(),
                asset_out: asset_out.clone(),
                lockup_until: None,
                expiration,
                referral: None,
            })
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
        .user2
        .swap_intent_action(
            env.swap_intent.id(),
            asset_out.asset().clone(),
            SwapIntentAction::Execute(ExecuteSwapIntentAction {
                id: intent_id.clone(),
                recipient: match asset_in {
                    Asset::Near(_) => GenericAccount::Near(env.user2.id().clone()),
                    Asset::CrossChain(_) => GenericAccount::CrossChain("user2".to_string()),
                },
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
        .is_executed(),);

    // skip check for same assets
    if match (&asset_in, &asset_out.asset()) {
        (Asset::Near(NearAsset::Native { .. }), Asset::Near(NearAsset::Native { .. })) => true,
        (
            Asset::Near(NearAsset::Nep141(FtAmount {
                token: token_in, ..
            })),
            Asset::Near(NearAsset::Nep141(FtAmount {
                token: token_out, ..
            })),
        ) => token_in == token_out,
        _ => false,
    } {
        return;
    }

    for (owner, asset) in [(&env.user2, &asset_in), (&env.user1, &asset_out.asset())] {
        match asset {
            Asset::Near(NearAsset::Native { amount }) => {
                assert!(
                    owner.view_account().await.unwrap().balance
                        >
                        // initial balance
                        NearToken::from_near(10)
                            .saturating_add(*amount)
                            // reserve some NEAR for gas
                            .saturating_sub(NearToken::from_near(1))
                );
            }
            Asset::Near(NearAsset::Nep141(FtAmount { token, amount })) => {
                assert_eq!(
                    owner.ft_token_balance_of(token, owner.id()).await.unwrap(),
                    amount.0,
                );
            }
            Asset::Near(NearAsset::Nep171(NftItem {
                collection,
                token_id,
            })) => {
                assert_eq!(
                    &env.root_account()
                        .nft_token(collection, token_id)
                        .await
                        .unwrap()
                        .unwrap()
                        .owner_id,
                    owner.id(),
                );
            }
            Asset::CrossChain(_) => {
                // we can't check the owner of cross-chain asset,
                // as we don't have oracles yet
            }
        }
    }
}
