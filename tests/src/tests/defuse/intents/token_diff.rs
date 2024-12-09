use std::{collections::BTreeMap, time::Duration};

use defuse::core::{
    fees::Pips,
    intents::{
        token_diff::{TokenDeltas, TokenDiff},
        DefuseIntents,
    },
    payload::multi::MultiPayload,
    tokens::TokenId,
    Deadline,
};
use near_sdk::AccountId;
use near_workspaces::Account;
use rand::{thread_rng, Rng};
use rstest::rstest;

use crate::{
    tests::defuse::{env::Env, DefuseSigner},
    utils::mt::MtExt,
};

use super::ExecuteIntentsExt;

#[rstest]
#[tokio::test]
async fn test_swap_p2p(#[values(Pips::ZERO, Pips::ONE_BIP, Pips::ONE_PERCENT)] fee: Pips) {
    let env = Env::builder().fee(fee).build().await;

    let ft1_token_id = TokenId::Nep141(env.ft1.clone());
    let ft2_token_id = TokenId::Nep141(env.ft2.clone());

    test_ft_diffs(
        &env,
        [
            AccountFtDiff {
                account: &env.user1,
                init_balances: [(&env.ft1, 100)].into_iter().collect(),
                diff: [TokenDeltas::default()
                    .with_add_deltas([
                        (ft1_token_id.clone(), -100),
                        (
                            ft2_token_id.clone(),
                            TokenDiff::closure_delta(&ft2_token_id, -200, fee).unwrap(),
                        ),
                    ])
                    .unwrap()]
                .into(),
                result_balances: [(
                    &env.ft2,
                    TokenDiff::closure_delta(&ft2_token_id, -200, fee).unwrap(),
                )]
                .into_iter()
                .collect(),
            },
            AccountFtDiff {
                account: &env.user2,
                init_balances: [(&env.ft2, 200)].into_iter().collect(),
                diff: [TokenDeltas::default()
                    .with_add_deltas([
                        (
                            ft1_token_id.clone(),
                            TokenDiff::closure_delta(&ft1_token_id, -100, fee).unwrap(),
                        ),
                        (ft2_token_id.clone(), -200),
                    ])
                    .unwrap()]
                .into(),
                result_balances: [(
                    &env.ft1,
                    TokenDiff::closure_delta(&ft1_token_id, -100, fee).unwrap(),
                )]
                .into_iter()
                .collect(),
            },
        ]
        .into(),
    )
    .await;
}

#[rstest]
#[tokio::test]
async fn test_swap_many(#[values(Pips::ZERO, Pips::ONE_BIP, Pips::ONE_PERCENT)] fee: Pips) {
    let env = Env::builder().fee(fee).build().await;

    let ft1_token_id = TokenId::Nep141(env.ft1.clone());
    let ft2_token_id = TokenId::Nep141(env.ft2.clone());
    let ft3_token_id = TokenId::Nep141(env.ft3.clone());

    test_ft_diffs(
        &env,
        [
            AccountFtDiff {
                account: &env.user1,
                init_balances: [(&env.ft1, 100)].into_iter().collect(),
                diff: [TokenDeltas::default()
                    .with_add_deltas([(ft1_token_id.clone(), -100), (ft2_token_id.clone(), 200)])
                    .unwrap()]
                .into(),
                result_balances: [(&env.ft2, 200)].into_iter().collect(),
            },
            AccountFtDiff {
                account: &env.user2,
                init_balances: [(&env.ft2, 1000)].into_iter().collect(),
                diff: [
                    TokenDeltas::default()
                        .with_add_deltas([
                            (
                                ft1_token_id.clone(),
                                TokenDiff::closure_delta(&ft1_token_id, -100, fee).unwrap(),
                            ),
                            (
                                ft2_token_id.clone(),
                                TokenDiff::closure_delta(&ft2_token_id, 200, fee).unwrap(),
                            ),
                        ])
                        .unwrap(),
                    TokenDeltas::default()
                        .with_add_deltas([
                            (
                                ft2_token_id.clone(),
                                TokenDiff::closure_delta(&ft2_token_id, 300, fee).unwrap(),
                            ),
                            (
                                ft3_token_id.clone(),
                                TokenDiff::closure_delta(&ft3_token_id, -500, fee).unwrap(),
                            ),
                        ])
                        .unwrap(),
                ]
                .into(),
                result_balances: [
                    (
                        &env.ft1,
                        TokenDiff::closure_delta(&ft1_token_id, -100, fee).unwrap(),
                    ),
                    (
                        &env.ft2,
                        1000 + TokenDiff::closure_delta(&ft2_token_id, 200, fee).unwrap()
                            + TokenDiff::closure_delta(&ft2_token_id, 300, fee).unwrap(),
                    ),
                    (
                        &env.ft3,
                        TokenDiff::closure_delta(&ft3_token_id, -500, fee).unwrap(),
                    ),
                ]
                .into_iter()
                .collect(),
            },
            AccountFtDiff {
                account: &env.user3,
                init_balances: [(&env.ft3, 500)].into_iter().collect(),
                diff: [TokenDeltas::default()
                    .with_add_deltas([(ft2_token_id.clone(), 300), (ft3_token_id.clone(), -500)])
                    .unwrap()]
                .into(),
                result_balances: [(&env.ft2, 300)].into_iter().collect(),
            },
        ]
        .into(),
    )
    .await;
}

type FtBalances<'a> = BTreeMap<&'a AccountId, i128>;

#[derive(Debug)]
struct AccountFtDiff<'a> {
    account: &'a Account,
    init_balances: FtBalances<'a>,
    diff: Vec<TokenDeltas>,
    result_balances: FtBalances<'a>,
}

async fn test_ft_diffs(env: &Env, accounts: Vec<AccountFtDiff<'_>>) {
    // deposit
    for account in &accounts {
        for (token_id, balance) in &account.init_balances {
            env.defuse_ft_mint(
                token_id,
                (*balance).try_into().unwrap(),
                account.account.id(),
            )
            .await
            .unwrap();
        }
    }

    let signed: Vec<MultiPayload> = accounts
        .iter()
        .flat_map(move |account| {
            account.diff.iter().cloned().map(|diff| {
                account.account.sign_defuse_message(
                    env.defuse.id(),
                    thread_rng().gen(),
                    Deadline::timeout(Duration::from_secs(120)),
                    DefuseIntents {
                        intents: [TokenDiff { diff }.into()].into(),
                    },
                )
            })
        })
        .collect();

    // simulate
    env.defuse
        .simulate_intents(signed.clone())
        .await
        .unwrap()
        .into_result()
        .unwrap();

    // verify
    env.defuse.execute_intents(signed).await.unwrap();

    // check balances
    for account in accounts {
        let (tokens, balances): (Vec<_>, Vec<_>) = account
            .result_balances
            .into_iter()
            .map(|(t, b)| {
                (
                    TokenId::Nep141(t.clone()).to_string(),
                    u128::try_from(b).unwrap(),
                )
            })
            .unzip();
        assert_eq!(
            env.mt_contract_batch_balance_of(env.defuse.id(), account.account.id(), &tokens)
                .await
                .unwrap(),
            balances
        );
    }
}

#[tokio::test]
async fn test_invariant_violated() {
    let env = Env::new().await;

    let ft1 = TokenId::Nep141(env.ft1.clone());
    let ft2 = TokenId::Nep141(env.ft2.clone());

    // deposit
    env.defuse_ft_mint(&env.ft1, 1000, env.user1.id())
        .await
        .unwrap();
    env.defuse_ft_mint(&env.ft2, 2000, env.user2.id())
        .await
        .unwrap();

    let signed: Vec<_> = [
        env.user1.sign_defuse_message(
            env.defuse.id(),
            thread_rng().gen(),
            Deadline::MAX,
            DefuseIntents {
                intents: [TokenDiff {
                    diff: TokenDeltas::default()
                        .with_add_deltas([(ft1.clone(), -1000), (ft2.clone(), 2000)])
                        .unwrap(),
                }
                .into()]
                .into(),
            },
        ),
        env.user1.sign_defuse_message(
            env.defuse.id(),
            thread_rng().gen(),
            Deadline::MAX,
            DefuseIntents {
                intents: [TokenDiff {
                    diff: TokenDeltas::default()
                        .with_add_deltas([(ft1.clone(), 1000), (ft2.clone(), -1999)])
                        .unwrap(),
                }
                .into()]
                .into(),
            },
        ),
    ]
    .into();

    assert_eq!(
        env.defuse
            .simulate_intents(signed.clone())
            .await
            .unwrap()
            .invariant_violated
            .unwrap()
            .into_unmatched_deltas(),
        Some(TokenDeltas::new([(ft2.clone(), 1)].into_iter().collect()))
    );

    env.defuse.execute_intents(signed).await.unwrap_err();

    // balances should stay the same
    assert_eq!(
        env.mt_contract_batch_balance_of(
            env.defuse.id(),
            env.user1.id(),
            [&ft1.to_string(), &ft2.to_string()]
        )
        .await
        .unwrap(),
        [1000, 0]
    );
    assert_eq!(
        env.mt_contract_batch_balance_of(
            env.defuse.id(),
            env.user2.id(),
            [&ft1.to_string(), &ft2.to_string()]
        )
        .await
        .unwrap(),
        [0, 2000]
    );
}
