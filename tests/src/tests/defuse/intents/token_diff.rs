use std::collections::BTreeMap;

use defuse_contracts::{
    defuse::{
        intents::{token_diff::TokenDiff, DefuseIntents},
        message::SignedDefuseMessage,
        tokens::{TokenAmounts, TokenId},
    },
    utils::{fees::Pips, Deadline},
};
use near_sdk::AccountId;
use near_workspaces::Account;
use rand::{thread_rng, Rng};
use serde_json::json;

use crate::{
    tests::defuse::{accounts::AccountManagerExt, env::Env, DefuseSigner},
    utils::mt::MtExt,
};

#[tokio::test]
async fn test_swap_p2p() {
    let env = Env::new().await.unwrap();
    test_ft_diffs(
        &env,
        [
            AccountFtDiff {
                account: &env.user1,
                init_balances: [(env.ft1.id(), 100)].into_iter().collect(),
                diff: [TokenAmounts::<i128>::default()
                    .with_try_extend::<i128>([
                        (TokenId::Nep141(env.ft1.id().clone()), -100),
                        (TokenId::Nep141(env.ft2.id().clone()), 200),
                    ])
                    .unwrap()]
                .into(),
                result_balances: [(env.ft2.id(), 200)].into_iter().collect(),
            },
            AccountFtDiff {
                account: &env.user2,
                init_balances: [(env.ft2.id(), 200)].into_iter().collect(),
                diff: [TokenAmounts::<i128>::default()
                    .with_try_extend::<i128>([
                        (TokenId::Nep141(env.ft1.id().clone()), 100),
                        (TokenId::Nep141(env.ft2.id().clone()), -200),
                    ])
                    .unwrap()]
                .into(),
                result_balances: [(env.ft1.id(), 100)].into_iter().collect(),
            },
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
            AccountFtDiff {
                account: &env.user1,
                init_balances: [(env.ft1.id(), 100)].into_iter().collect(),
                diff: [TokenAmounts::<i128>::default()
                    .with_try_extend::<i128>([
                        (TokenId::Nep141(env.ft1.id().clone()), -100),
                        (TokenId::Nep141(env.ft2.id().clone()), 200),
                    ])
                    .unwrap()]
                .into(),
                result_balances: [(env.ft2.id(), 200)].into_iter().collect(),
            },
            AccountFtDiff {
                account: &env.user2,
                init_balances: [(env.ft2.id(), 500)].into_iter().collect(),
                diff: [
                    TokenAmounts::<i128>::default()
                        .with_try_extend::<i128>([
                            (TokenId::Nep141(env.ft1.id().clone()), 100),
                            (TokenId::Nep141(env.ft2.id().clone()), -200),
                        ])
                        .unwrap(),
                    TokenAmounts::<i128>::default()
                        .with_try_extend::<i128>([
                            (TokenId::Nep141(env.ft3.id().clone()), 500),
                            (TokenId::Nep141(env.ft2.id().clone()), -300),
                        ])
                        .unwrap(),
                ]
                .into(),
                result_balances: [(env.ft1.id(), 100), (env.ft2.id(), 0), (env.ft3.id(), 500)]
                    .into_iter()
                    .collect(),
            },
            AccountFtDiff {
                account: &env.user3,
                init_balances: [(env.ft3.id(), 500)].into_iter().collect(),
                diff: [TokenAmounts::<i128>::default()
                    .with_try_extend::<i128>([
                        (TokenId::Nep141(env.ft2.id().clone()), 300),
                        (TokenId::Nep141(env.ft3.id().clone()), -500),
                    ])
                    .unwrap()]
                .into(),
                result_balances: [(env.ft2.id(), 300)].into_iter().collect(),
            },
        ]
        .into(),
    )
    .await;
}

#[tokio::test]
async fn test_swap_many_with_fees() {
    let fee = Pips::ONE_PERCENT * 2;
    let env = Env::builder().with_fee(fee).build().await.unwrap();
    test_ft_diffs(
        &env,
        [
            AccountFtDiff {
                account: &env.user1,
                init_balances: [(env.ft1.id(), 100)].into_iter().collect(),
                diff: [TokenAmounts::<i128>::default()
                    .with_try_extend::<i128>([
                        (TokenId::Nep141(env.ft1.id().clone()), -100),
                        (TokenId::Nep141(env.ft2.id().clone()), 200),
                    ])
                    .unwrap()]
                .into(),
                result_balances: [(env.ft2.id(), 200)].into_iter().collect(),
            },
            AccountFtDiff {
                account: &env.user2,
                init_balances: [(env.ft2.id(), 520)].into_iter().collect(),
                diff: [
                    TokenAmounts::<i128>::default()
                        .with_try_extend::<i128>([
                            (TokenId::Nep141(env.ft1.id().clone()), 97),
                            (TokenId::Nep141(env.ft2.id().clone()), -208),
                        ])
                        .unwrap(),
                    TokenAmounts::<i128>::default()
                        .with_try_extend::<i128>([
                            (TokenId::Nep141(env.ft3.id().clone()), 481),
                            (TokenId::Nep141(env.ft2.id().clone()), -312),
                        ])
                        .unwrap(),
                ]
                .into(),
                result_balances: [(env.ft1.id(), 97), (env.ft2.id(), 0), (env.ft3.id(), 481)]
                    .into_iter()
                    .collect(),
            },
            AccountFtDiff {
                account: &env.user3,
                init_balances: [(env.ft3.id(), 500)].into_iter().collect(),
                diff: [TokenAmounts::<i128>::default()
                    .with_try_extend::<i128>([
                        (TokenId::Nep141(env.ft2.id().clone()), 300),
                        (TokenId::Nep141(env.ft3.id().clone()), -500),
                    ])
                    .unwrap()]
                .into(),
                result_balances: [(env.ft2.id(), 300)].into_iter().collect(),
            },
        ]
        .into(),
    )
    .await;
}

type FtBalances<'a> = BTreeMap<&'a AccountId, u128>;

#[derive(Debug)]
struct AccountFtDiff<'a> {
    account: &'a Account,
    init_balances: FtBalances<'a>,
    diff: Vec<TokenAmounts<i128>>,
    result_balances: FtBalances<'a>,
}

async fn test_ft_diffs(env: &Env, accounts: Vec<AccountFtDiff<'_>>) {
    // deposit
    for account in &accounts {
        for (token_id, balance) in &account.init_balances {
            env.defuse_ft_mint(token_id, *balance, account.account.id())
                .await
                .unwrap();
        }
    }

    // verify
    env.defuse
        .execute_signed_intents(accounts.iter().flat_map(move |account| {
            account.diff.iter().cloned().map(|diff| {
                account.account.sign_defuse_message(
                    env.defuse.id(),
                    DefuseIntents {
                        intents: [TokenDiff { diff }.into()].into(),
                    },
                    thread_rng().gen(),
                    Deadline::infinity(),
                )
            })
        }))
        .await
        .unwrap();

    // check balances
    for account in accounts {
        let (tokens, balances): (Vec<_>, Vec<_>) = account
            .result_balances
            .into_iter()
            .map(|(t, b)| (TokenId::Nep141(t.clone()).to_string(), b))
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
        .execute_signed_intents([
            env.user1.sign_defuse_message(
                env.defuse.id(),
                DefuseIntents {
                    intents: [TokenDiff {
                        diff: TokenAmounts::default()
                            .with_try_extend::<i128>([(ft1.clone(), -1000), (ft2.clone(), 2000)])
                            .unwrap(),
                    }
                    .into()]
                    .into(),
                },
                thread_rng().gen(),
                Deadline::infinity(),
            ),
            env.user1.sign_defuse_message(
                env.defuse.id(),
                DefuseIntents {
                    intents: [TokenDiff {
                        diff: TokenAmounts::default()
                            .with_try_extend::<i128>([(ft1.clone(), 1000), (ft2.clone(), -1999)])
                            .unwrap(),
                    }
                    .into()]
                    .into(),
                },
                thread_rng().gen(),
                Deadline::infinity(),
            ),
        ])
        .await
        .unwrap_err();

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

pub trait SignedIntentsExt: AccountManagerExt {
    async fn execute_signed_intents(
        &self,
        intents: impl IntoIterator<Item = SignedDefuseMessage<DefuseIntents>>,
    ) -> anyhow::Result<()>;
}

impl SignedIntentsExt for near_workspaces::Account {
    async fn execute_signed_intents(
        &self,
        intents: impl IntoIterator<Item = SignedDefuseMessage<DefuseIntents>>,
    ) -> anyhow::Result<()> {
        self.call(self.id(), "execute_signed_intents")
            .args_json(json!({
                "signed": intents.into_iter().collect::<Vec<_>>(),
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()
            .map(|_| ())
            .map_err(Into::into)
    }
}

impl SignedIntentsExt for near_workspaces::Contract {
    async fn execute_signed_intents(
        &self,
        intents: impl IntoIterator<Item = SignedDefuseMessage<DefuseIntents>>,
    ) -> anyhow::Result<()> {
        self.as_account().execute_signed_intents(intents).await
    }
}
