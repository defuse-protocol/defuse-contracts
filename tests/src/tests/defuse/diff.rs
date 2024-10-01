use std::collections::BTreeMap;

use defuse_contracts::{
    crypto::SignedPayload,
    defuse::{
        diff::{tokens::TokenDeltas, AccountDiff},
        payload::{MultiStandardPayload, SignedPayloads},
        tokens::TokenId,
    },
    nep413::Nep413Payload,
};
use near_sdk::AccountId;
use near_workspaces::Account;
use rand::{thread_rng, Rng};
use serde_json::json;

use crate::utils::crypto::Signer;

use super::{accounts::AccountManagerExt, env::Env, DefuseExt};

#[tokio::test]
async fn test_swap_p2p() {
    let env = Env::new().await.unwrap();
    test_ft_diffs(
        &env,
        [
            (
                &env.user1,
                AccountFtDiff {
                    init_balances: [(env.ft1.id(), 100)].into_iter().collect(),
                    deltas: [TokenDeltas::default()
                        .with_add_delta(env.ft1.id(), -100)
                        .unwrap()
                        .with_add_delta(env.ft2.id(), 200)
                        .unwrap()]
                    .into(),
                    result_balances: [(env.ft2.id(), 200)].into_iter().collect(),
                },
            ),
            (
                &env.user2,
                AccountFtDiff {
                    init_balances: [(env.ft2.id(), 200)].into_iter().collect(),
                    deltas: [TokenDeltas::default()
                        .with_add_delta(env.ft1.id(), 100)
                        .unwrap()
                        .with_add_delta(env.ft2.id(), -200)
                        .unwrap()]
                    .into(),
                    result_balances: [(env.ft1.id(), 100)].into_iter().collect(),
                },
            ),
        ]
        .into(),
    )
    .await;
}

#[tokio::test]
async fn test_swap_many() {
    let env = Env::new().await.unwrap();
    test_ft_diffs(
        &env,
        [
            (
                &env.user1,
                AccountFtDiff {
                    init_balances: [(env.ft1.id(), 100)].into_iter().collect(),
                    deltas: [TokenDeltas::default()
                        .with_add_delta(env.ft1.id(), -100)
                        .unwrap()
                        .with_add_delta(env.ft2.id(), 200)
                        .unwrap()]
                    .into(),
                    result_balances: [(env.ft2.id(), 200)].into_iter().collect(),
                },
            ),
            (
                &env.user2,
                AccountFtDiff {
                    init_balances: [(env.ft2.id(), 500)].into_iter().collect(),
                    deltas: [
                        TokenDeltas::default()
                            .with_add_delta(env.ft1.id(), 100)
                            .unwrap()
                            .with_add_delta(env.ft2.id(), -200)
                            .unwrap(),
                        TokenDeltas::default()
                            .with_add_delta(env.ft3.id(), 500)
                            .unwrap()
                            .with_add_delta(env.ft2.id(), -300)
                            .unwrap(),
                    ]
                    .into(),
                    result_balances: [(env.ft1.id(), 100), (env.ft2.id(), 0), (env.ft3.id(), 500)]
                        .into_iter()
                        .collect(),
                },
            ),
            (
                &env.user3,
                AccountFtDiff {
                    init_balances: [(env.ft3.id(), 500)].into_iter().collect(),
                    deltas: [TokenDeltas::default()
                        .with_add_delta(env.ft2.id(), 300)
                        .unwrap()
                        .with_add_delta(env.ft3.id(), -500)
                        .unwrap()]
                    .into(),
                    result_balances: [(env.ft2.id(), 300)].into_iter().collect(),
                },
            ),
        ]
        .into(),
    )
    .await;
}

type FtBalances<'a> = BTreeMap<&'a AccountId, u128>;

#[derive(Debug)]
struct AccountFtDiff<'a> {
    init_balances: FtBalances<'a>,
    deltas: Vec<TokenDeltas<&'a AccountId>>,
    result_balances: FtBalances<'a>,
}

async fn test_ft_diffs(env: &Env, accounts: Vec<(&Account, AccountFtDiff<'_>)>) {
    // deposit
    for (account, t) in &accounts {
        for (token_id, balance) in &t.init_balances {
            env.defuse_ft_mint(token_id, *balance, account.id())
                .await
                .unwrap();
        }
    }

    // verify
    env.defuse
        .apply_signed_diffs(accounts.iter().map(move |(account, t)| {
            (
                account.id(),
                t.deltas.iter().map(|deltas| {
                    account.sign_payload(
                        Nep413Payload::new(
                            AccountDiff::default()
                                .with_tokens(
                                    deltas
                                        .iter()
                                        .map(|(t, d)| (TokenId::Nep141((*t).clone()), *d)),
                                )
                                .unwrap(),
                        )
                        .with_nonce(thread_rng().gen())
                        .with_recipient(env.defuse.id())
                        .into(),
                    )
                }),
            )
        }))
        .await
        .unwrap();

    // check balances
    for (account, t) in accounts {
        let (tokens, balances): (Vec<_>, Vec<_>) = t
            .result_balances
            .into_iter()
            .map(|(t, b)| (TokenId::Nep141(t.clone()), b))
            .unzip();
        assert_eq!(
            env.defuse
                .mt_batch_balance_of(account.id(), &tokens)
                .await
                .unwrap(),
            balances
        );
    }
}

#[tokio::test]
async fn test_invariant_violated() {
    let env = Env::new().await.unwrap();

    let ft1 = TokenId::Nep141(env.ft1.id().clone());
    let ft2 = TokenId::Nep141(env.ft2.id().clone());

    // deposit
    env.defuse_ft_mint(env.ft1.id(), 1000, env.user1.id())
        .await
        .unwrap();
    env.defuse_ft_mint(env.ft2.id(), 2000, env.user2.id())
        .await
        .unwrap();

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
                            .with_tokens([(ft1.clone(), 1000), (ft2.clone(), -1999)])
                            .unwrap(),
                    )
                    .with_recipient(env.defuse.id())
                    .into(),
                )],
            ),
        ])
        .await
        .unwrap_err();

    // balances should stay the same
    assert_eq!(
        env.defuse
            .mt_batch_balance_of(env.user1.id(), [&ft1, &ft2])
            .await
            .unwrap(),
        [1000, 0]
    );
    assert_eq!(
        env.defuse
            .mt_batch_balance_of(env.user2.id(), [&ft1, &ft2])
            .await
            .unwrap(),
        [0, 2000]
    );
}

pub trait SignedDifferExt: AccountManagerExt {
    async fn apply_signed_diffs(
        &self,
        diffs: impl IntoIterator<
            Item = (
                &AccountId,
                impl IntoIterator<Item = SignedPayload<MultiStandardPayload<AccountDiff>>>,
            ),
        >,
    ) -> anyhow::Result<()>;
}

impl SignedDifferExt for near_workspaces::Account {
    async fn apply_signed_diffs(
        &self,
        diffs: impl IntoIterator<
            Item = (
                &AccountId,
                impl IntoIterator<Item = SignedPayload<MultiStandardPayload<AccountDiff>>>,
            ),
        >,
    ) -> anyhow::Result<()> {
        self.call(self.id(), "apply_signed_diffs")
            .args_json(json!({
                "diffs": diffs
                    .into_iter()
                    .map(|(account_id, diffs)| (
                        account_id.clone(),
                        diffs.into_iter().collect(),
                    )).collect::<SignedPayloads<_>>(),
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()
            .map(|_| ())
            .map_err(Into::into)
    }
}

impl SignedDifferExt for near_workspaces::Contract {
    async fn apply_signed_diffs(
        &self,
        diffs: impl IntoIterator<
            Item = (
                &AccountId,
                impl IntoIterator<Item = SignedPayload<MultiStandardPayload<AccountDiff>>>,
            ),
        >,
    ) -> anyhow::Result<()> {
        self.as_account().apply_signed_diffs(diffs).await
    }
}
