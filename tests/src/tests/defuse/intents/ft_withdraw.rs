use defuse_contracts::{
    defuse::{
        intents::{tokens::FtWithdraw, DefuseIntents},
        tokens::TokenId,
    },
    utils::Deadline,
};
use near_sdk::{AccountId, NearToken};
use rand::{thread_rng, Rng};

use super::ExecuteIntentsExt;
use crate::{
    tests::defuse::{env::Env, tokens::nep141::DefuseFtReceiver, DefuseSigner},
    utils::{
        ft::{FtExt, FT_STORAGE_DEPOSIT},
        mt::MtExt,
        wnear::WNearExt,
    },
};

#[tokio::test]
async fn test_withdraw_intent() {
    let env = Env::new().await.unwrap();

    env.defuse_ft_mint(&env.ft1, 1000, env.user1.id())
        .await
        .unwrap();

    let other_user_id: AccountId = "other-user.near".parse().unwrap();

    env.defuse
        .execute_intents([env.user1.sign_defuse_message(
            env.defuse.id(),
            thread_rng().gen(),
            Deadline::infinity(),
            DefuseIntents {
                intents: [FtWithdraw {
                    token: env.ft1.clone(),
                    receiver_id: other_user_id.clone(),
                    amount: 1000.into(),
                    memo: None,
                    storage_deposit: None,
                }
                .into()]
                .into(),
            },
        )])
        .await
        .unwrap();

    let ft1 = TokenId::Nep141(env.ft1.clone());
    assert_eq!(
        env.mt_contract_balance_of(env.defuse.id(), env.user1.id(), &ft1.to_string())
            .await
            .unwrap(),
        1000
    );
    assert_eq!(
        env.ft_token_balance_of(&env.ft1, &other_user_id)
            .await
            .unwrap(),
        0
    );

    env.defuse
        .execute_intents([env.user1.sign_defuse_message(
            env.defuse.id(),
            thread_rng().gen(),
            Deadline::infinity(),
            DefuseIntents {
                intents: [FtWithdraw {
                    token: env.ft1.clone(),
                    receiver_id: other_user_id.clone(),
                    amount: 1000.into(),
                    memo: None,
                    // user has no wnear yet
                    storage_deposit: Some(FT_STORAGE_DEPOSIT),
                }
                .into()]
                .into(),
            },
        )])
        .await
        .unwrap_err();

    // wrap NEAR
    env.user1
        .near_deposit(env.wnear.id(), NearToken::from_near(1))
        .await
        .unwrap();
    // deposit wNEAR
    env.user1
        .defuse_ft_deposit(
            env.defuse.id(),
            env.wnear.id(),
            NearToken::from_near(1).as_yoctonear(),
            None,
        )
        .await
        .unwrap();

    env.defuse
        .execute_intents([env.user1.sign_defuse_message(
            env.defuse.id(),
            thread_rng().gen(),
            Deadline::infinity(),
            DefuseIntents {
                intents: [FtWithdraw {
                    token: env.ft1.clone(),
                    receiver_id: other_user_id.clone(),
                    amount: 1000.into(),
                    memo: None,
                    // now user has wNEAR to pay for it
                    storage_deposit: Some(FT_STORAGE_DEPOSIT),
                }
                .into()]
                .into(),
            },
        )])
        .await
        .unwrap();

    assert_eq!(
        env.mt_contract_balance_of(env.defuse.id(), env.user1.id(), &ft1.to_string())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.mt_contract_balance_of(
            env.defuse.id(),
            env.user1.id(),
            &TokenId::Nep141(env.wnear.id().clone()).to_string()
        )
        .await
        .unwrap(),
        NearToken::from_near(1)
            .saturating_sub(FT_STORAGE_DEPOSIT)
            .as_yoctonear(),
    );

    assert_eq!(
        env.ft_token_balance_of(&env.ft1, &other_user_id)
            .await
            .unwrap(),
        1000
    );
}
