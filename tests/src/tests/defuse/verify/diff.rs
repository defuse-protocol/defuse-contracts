use defuse_contracts::{
    defuse::{token::TokenId, verify::diff::AccountDiff},
    nep413::Nep413Payload,
};

use crate::{
    tests::defuse::{env::Env, verify::VerifierExt, DefuseExt},
    utils::crypto::Signer,
};

#[tokio::test]
async fn test_diff() {
    let env = Env::new().await.unwrap();

    let ft1 = TokenId::Nep141(env.ft1.id().clone());
    let ft2 = TokenId::Nep141(env.ft2.id().clone());

    // deposit
    env.defuse_ft_mint(env.ft1.id(), 1000, env.user1.id().clone(), [].into())
        .await
        .unwrap();
    env.defuse_ft_mint(env.ft2.id(), 2000, env.user2.id().clone(), [].into())
        .await
        .unwrap();

    // verify
    env.defuse
        .apply_signed_diffs([
            (
                env.user1.id(),
                [env.user1.sign_payload(
                    Nep413Payload::new(
                        AccountDiff::default()
                            .with_tokens([(ft1.clone(), -1000), (ft2.clone(), 2000)])
                            .unwrap(),
                    )
                    .with_recipient(env.defuse.id())
                    .into(),
                )],
            ),
            (
                env.user2.id(),
                [env.user2.sign_payload(
                    Nep413Payload::new(
                        AccountDiff::default()
                            .with_tokens([(ft1.clone(), 1000), (ft2.clone(), -2000)])
                            .unwrap(),
                    )
                    .with_recipient(env.defuse.id())
                    .into(),
                )],
            ),
        ])
        .await
        .unwrap();

    // check balances
    assert_eq!(
        env.defuse
            .mt_batch_balance_of(env.user1.id(), [&ft1, &ft2])
            .await
            .unwrap(),
        [0, 2000]
    );
    assert_eq!(
        env.defuse
            .mt_batch_balance_of(env.user2.id(), [&ft1, &ft2])
            .await
            .unwrap(),
        [1000, 0]
    );
}
