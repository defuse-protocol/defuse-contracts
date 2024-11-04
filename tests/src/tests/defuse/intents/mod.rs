use defuse_contracts::{
    crypto::SignedPayload,
    defuse::{
        intents::{tokens::MtBatchTransfer, DefuseIntents},
        payload::{multi::MultiStandardPayload, DefusePayload},
        tokens::TokenId,
    },
    utils::Deadline,
};
use near_sdk::{json_types::U128, AccountId};
use rand::{thread_rng, Rng};
use serde_json::json;

use crate::utils::mt::MtExt;

use super::{accounts::AccountManagerExt, env::Env};

mod ft_withdraw;
mod relayers;
mod token_diff;

pub trait ExecuteIntentsExt: AccountManagerExt {
    async fn defuse_execute_intents(
        &self,
        defuse_id: &AccountId,
        intents: impl IntoIterator<Item = SignedPayload<MultiStandardPayload>>,
    ) -> anyhow::Result<()>;
    async fn execute_intents(
        &self,
        intents: impl IntoIterator<Item = SignedPayload<MultiStandardPayload>>,
    ) -> anyhow::Result<()>;
}

impl ExecuteIntentsExt for near_workspaces::Account {
    async fn defuse_execute_intents(
        &self,
        defuse_id: &AccountId,
        intents: impl IntoIterator<Item = SignedPayload<MultiStandardPayload>>,
    ) -> anyhow::Result<()> {
        let args = json!({
            "intents": intents.into_iter().collect::<Vec<_>>(),
        });
        println!(
            "execute_intents({})",
            serde_json::to_string_pretty(&args).unwrap()
        );
        self.call(defuse_id, "execute_intents")
            .args_json(args)
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
    async fn execute_intents(
        &self,
        intents: impl IntoIterator<Item = SignedPayload<MultiStandardPayload>>,
    ) -> anyhow::Result<()> {
        self.defuse_execute_intents(self.id(), intents).await
    }
}

impl ExecuteIntentsExt for near_workspaces::Contract {
    async fn defuse_execute_intents(
        &self,
        defuse_id: &AccountId,
        intents: impl IntoIterator<Item = SignedPayload<MultiStandardPayload>>,
    ) -> anyhow::Result<()> {
        self.as_account()
            .defuse_execute_intents(defuse_id, intents)
            .await
    }
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

    let ft1 = TokenId::Nep141(env.ft1.clone());

    // deposit
    env.defuse_ft_mint(&env.ft1, 1000, env.user1.id())
        .await
        .unwrap();

    // ignore the output
    let _ = env
        .defuse
        .call("simulate_intents")
        .args_json(json!({
            "intents": [DefusePayload {
                signer_id: env.user1.id().clone(),
                verifying_contract: env.defuse.id().clone(),
                deadline: Deadline::infinity(),
                nonce: thread_rng().gen(),
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
