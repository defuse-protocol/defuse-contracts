use defuse::{
    core::{
        intents::{tokens::Transfer, DefuseIntents},
        payload::multi::MultiPayload,
        tokens::{TokenAmounts, TokenId},
        Deadline,
    },
    intents::SimulationOutput,
};
use near_sdk::AccountId;
use rand::{thread_rng, Rng};
use serde_json::json;

use crate::utils::mt::MtExt;

use super::{accounts::AccountManagerExt, env::Env, DefuseSigner};

mod ft_withdraw;
mod relayers;
mod token_diff;

pub trait ExecuteIntentsExt: AccountManagerExt {
    async fn defuse_execute_intents(
        &self,
        defuse_id: &AccountId,
        intents: impl IntoIterator<Item = MultiPayload>,
    ) -> anyhow::Result<()>;
    async fn execute_intents(
        &self,
        intents: impl IntoIterator<Item = MultiPayload>,
    ) -> anyhow::Result<()>;

    async fn defuse_simulate_intents(
        &self,
        defuse_id: &AccountId,
        intents: impl IntoIterator<Item = MultiPayload>,
    ) -> anyhow::Result<SimulationOutput>;
    async fn simulate_intents(
        &self,
        intents: impl IntoIterator<Item = MultiPayload>,
    ) -> anyhow::Result<SimulationOutput>;
}

impl ExecuteIntentsExt for near_workspaces::Account {
    async fn defuse_execute_intents(
        &self,
        defuse_id: &AccountId,
        intents: impl IntoIterator<Item = MultiPayload>,
    ) -> anyhow::Result<()> {
        let args = json!({
            "signed": intents.into_iter().collect::<Vec<_>>(),
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
        intents: impl IntoIterator<Item = MultiPayload>,
    ) -> anyhow::Result<()> {
        self.defuse_execute_intents(self.id(), intents).await
    }

    async fn defuse_simulate_intents(
        &self,
        defuse_id: &AccountId,
        intents: impl IntoIterator<Item = MultiPayload>,
    ) -> anyhow::Result<SimulationOutput> {
        let args = json!({
            "signed": intents.into_iter().collect::<Vec<_>>(),
        });
        println!(
            "simulate_intents({})",
            serde_json::to_string_pretty(&args).unwrap()
        );
        self.view(defuse_id, "simulate_intents")
            .args_json(args)
            .await?
            .json()
            .map_err(Into::into)
    }
    async fn simulate_intents(
        &self,
        intents: impl IntoIterator<Item = MultiPayload>,
    ) -> anyhow::Result<SimulationOutput> {
        self.defuse_simulate_intents(self.id(), intents).await
    }
}

impl ExecuteIntentsExt for near_workspaces::Contract {
    async fn defuse_execute_intents(
        &self,
        defuse_id: &AccountId,
        intents: impl IntoIterator<Item = MultiPayload>,
    ) -> anyhow::Result<()> {
        self.as_account()
            .defuse_execute_intents(defuse_id, intents)
            .await
    }
    async fn execute_intents(
        &self,
        intents: impl IntoIterator<Item = MultiPayload>,
    ) -> anyhow::Result<()> {
        self.as_account().execute_intents(intents).await
    }

    async fn defuse_simulate_intents(
        &self,
        defuse_id: &AccountId,
        intents: impl IntoIterator<Item = MultiPayload>,
    ) -> anyhow::Result<SimulationOutput> {
        self.as_account()
            .defuse_simulate_intents(defuse_id, intents)
            .await
    }
    async fn simulate_intents(
        &self,
        intents: impl IntoIterator<Item = MultiPayload>,
    ) -> anyhow::Result<SimulationOutput> {
        self.as_account().simulate_intents(intents).await
    }
}

#[tokio::test]
async fn test_simulate_is_view_method() {
    let env = Env::new().await;

    let ft1 = TokenId::Nep141(env.ft1.clone());

    // deposit
    env.defuse_ft_mint(&env.ft1, 1000, env.user1.id())
        .await
        .unwrap();

    env.defuse
        .simulate_intents([env.user1.sign_defuse_message(
            env.defuse.id(),
            thread_rng().gen(),
            Deadline::MAX,
            DefuseIntents {
                intents: [Transfer {
                    receiver_id: env.user2.id().clone(),
                    tokens: TokenAmounts::new([(ft1.clone(), 1000)].into_iter().collect()),
                    memo: None,
                }
                .into()]
                .into(),
            },
        )])
        .await
        .unwrap()
        .into_result()
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
