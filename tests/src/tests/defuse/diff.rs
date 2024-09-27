use std::collections::HashMap;

use defuse_contracts::{
    crypto::SignedPayload,
    defuse::{diff::AccountDiff, payload::MultiStandardPayload, tokens::TokenId},
    nep413::Nep413Payload,
};
use near_sdk::AccountId;
use serde_json::json;

use crate::utils::crypto::Signer;

use super::{accounts::AccountManagerExt, env::Env, DefuseExt};

#[tokio::test]
async fn test_diff() {
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
                        diffs.into_iter().collect::<Vec<_>>(),
                    )).collect::<HashMap<_, _>>(),
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