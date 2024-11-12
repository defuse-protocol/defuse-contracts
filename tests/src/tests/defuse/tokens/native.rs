use defuse_contracts::{
    defuse::{
        intents::{tokens::NativeWithdraw, DefuseIntents, Intent},
        tokens::TokenId,
    },
    utils::Deadline,
};
use near_sdk::NearToken;
use rand::{thread_rng, Rng};

use crate::{
    tests::defuse::{
        env::Env, intents::ExecuteIntentsExt, tokens::nep141::DefuseFtReceiver, DefuseSigner,
    },
    utils::{mt::MtExt, wnear::WNearExt},
};

#[tokio::test]
async fn test_withdraw() {
    let env = Env::new().await.unwrap();

    const AMOUNT: NearToken = NearToken::from_near(100);
    let wnear_token_id = TokenId::Nep141(env.wnear.id().clone());

    env.transfer_near(env.user1.id(), AMOUNT)
        .await
        .unwrap()
        .into_result()
        .unwrap();

    env.user1
        .near_deposit(env.wnear.id(), AMOUNT)
        .await
        .unwrap();

    env.user1
        .defuse_ft_deposit(env.defuse.id(), env.wnear.id(), AMOUNT.as_yoctonear(), None)
        .await
        .unwrap();

    let old_balance = env.user2.view_account().await.unwrap().balance;

    env.defuse
        .execute_intents([env.user1.sign_defuse_message(
            env.defuse.id(),
            thread_rng().gen(),
            Deadline::infinity(),
            DefuseIntents {
                intents: [Intent::NativeWithdraw(NativeWithdraw {
                    receiver_id: env.user2.id().clone(),
                    amount: AMOUNT,
                })]
                .into(),
            },
        )])
        .await
        .unwrap();

    assert_eq!(
        env.mt_contract_balance_of(env.defuse.id(), env.user1.id(), &wnear_token_id.to_string())
            .await
            .unwrap(),
        0
    );

    let new_balance = env.user2.view_account().await.unwrap().balance;
    assert_eq!(old_balance.saturating_add(AMOUNT), new_balance);
}
