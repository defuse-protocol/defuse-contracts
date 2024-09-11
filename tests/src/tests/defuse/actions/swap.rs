use defuse_contracts::{
    defuse::{token::TokenId, verify::diff::AccountDiff},
    nep413::Payload,
};

use crate::{
    tests::defuse::{env::Env, verify::VerifierExt, DefuseExt},
    utils::{crypto::Signer, ft::FtExt},
};

#[tokio::test]
async fn test_swap() {
    let env = Env::new().await.unwrap();

    env.ft_storage_deposit(
        env.ft1.id(),
        &[env.user1.id(), env.user2.id(), env.defuse.id()],
    )
    .await
    .unwrap();
    env.ft_storage_deposit(
        env.ft2.id(),
        &[env.user1.id(), env.user2.id(), env.defuse.id()],
    )
    .await
    .unwrap();

    env.ft_mint(env.ft1.id(), env.user1.id(), 1000)
        .await
        .unwrap();
    env.ft_mint(env.ft2.id(), env.user2.id(), 2000)
        .await
        .unwrap();

    // deposit
    env.user1
        .ft_transfer_call(env.ft1.id(), env.defuse.id(), 1000, None, "")
        .await
        .unwrap();
    env.user2
        .ft_transfer_call(env.ft2.id(), env.defuse.id(), 2000, None, "")
        .await
        .unwrap();

    // sign
    let user1_signed = env.user1.sign_nep413(
        Payload::new(
            AccountDiff::default()
                .with_tokens([
                    (TokenId::Nep141(env.ft1.id().clone()), -1000),
                    (TokenId::Nep141(env.ft2.id().clone()), 2000),
                ])
                .unwrap(),
        )
        .with_recipient(env.defuse.id()),
    );
    let user2_signed = env.user2.sign_nep413(
        Payload::new(
            AccountDiff::default()
                .with_tokens([
                    (TokenId::Nep141(env.ft1.id().clone()), 1000),
                    (TokenId::Nep141(env.ft2.id().clone()), -2000),
                ])
                .unwrap(),
        )
        .with_recipient(env.defuse.id()),
    );

    // verify
    env.defuse
        .apply_signed_diffs(
            [
                (env.user1.id().clone(), [user1_signed].into()),
                (env.user2.id().clone(), [user2_signed].into()),
            ]
            .into_iter()
            .collect(),
        )
        .await
        .unwrap();

    assert_eq!(
        env.defuse
            .mt_balance_of(env.user1.id(), &TokenId::Nep141(env.ft1.id().clone()))
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.defuse
            .mt_balance_of(env.user1.id(), &TokenId::Nep141(env.ft2.id().clone()))
            .await
            .unwrap(),
        2000
    );

    assert_eq!(
        env.defuse
            .mt_balance_of(env.user2.id(), &TokenId::Nep141(env.ft1.id().clone()))
            .await
            .unwrap(),
        1000
    );
    assert_eq!(
        env.defuse
            .mt_balance_of(env.user2.id(), &TokenId::Nep141(env.ft2.id().clone()))
            .await
            .unwrap(),
        0
    );
}
