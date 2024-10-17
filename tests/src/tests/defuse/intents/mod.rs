use defuse_contracts::{
    crypto::SignedPayload,
    defuse::{
        intents::{tokens::MtBatchTransfer, DefuseIntents},
        payload::{DefuseMessage, MultiStandardPayload},
        tokens::TokenId,
    },
    utils::Deadline,
};
use near_sdk::json_types::U128;
use serde_json::json;

use crate::utils::mt::MtExt;

use super::{accounts::AccountManagerExt, env::Env};

mod token_diff;

pub trait ExecuteIntentsExt: AccountManagerExt {
    async fn execute_intents(
        &self,
        intents: impl IntoIterator<Item = SignedPayload<MultiStandardPayload>>,
    ) -> anyhow::Result<()>;
}

impl ExecuteIntentsExt for near_workspaces::Account {
    async fn execute_intents(
        &self,
        intents: impl IntoIterator<Item = SignedPayload<MultiStandardPayload>>,
    ) -> anyhow::Result<()> {
        self.call(self.id(), "execute_intents")
            .args_json(json!({
                "intents": intents.into_iter().collect::<Vec<_>>(),
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()
            .map(|outcome| {
                println!(
                    "execute_intents: total_gas_burnt: {}, logs: {:#?}",
                    outcome.total_gas_burnt,
                    outcome.logs()
                );
            })
            .map_err(Into::into)
    }
}

impl ExecuteIntentsExt for near_workspaces::Contract {
    async fn execute_intents(
        &self,
        intents: impl IntoIterator<Item = SignedPayload<MultiStandardPayload>>,
    ) -> anyhow::Result<()> {
        self.as_account().execute_intents(intents).await
    }
}

#[tokio::test]
async fn test_simulate_is_view_method() {
    let env = Env::new().await.unwrap();

    let ft1 = TokenId::Nep141(env.ft1.id().clone());

    // deposit
    env.defuse_ft_mint(env.ft1.id(), 1000, env.user1.id())
        .await
        .unwrap();

    // ignore the output
    let _ = env
        .defuse
        .call("simulate_intents")
        .args_json(json!({
            "intents": [DefuseMessage {
                signer_id: env.user1.id().clone(),
                deadline: Deadline::infinity(),
                message: DefuseIntents {
                    intents: [MtBatchTransfer {
                        receiver_id: env.user2.id().clone(),
                        token_ids: [ft1.clone()].into(),
                        amounts: [U128(1000)].into(),
                        memo: None,
                    }
                    .into()]
                    .into(),
                }}],
        }))
        .max_gas()
        .transact()
        .await
        .unwrap();

    assert_eq!(
        env.defuse
            .mt_balance_of(env.user1.id(), &ft1.to_string())
            .await
            .unwrap(),
        1000
    );
    assert_eq!(
        env.defuse
            .mt_balance_of(env.user2.id(), &ft1.to_string())
            .await
            .unwrap(),
        0
    );
}
