use defuse_contracts::intents::mpc::{Account, Action, BarterIntent, Intent, IntentID};
use lazy_static::lazy_static;
use near_sdk::{AccountId, NearToken};
use near_workspaces::Contract;
use serde_json::json;

use crate::{
    tests::account::{AccountContract, MpcAccount},
    utils::{account::AccountExt, read_wasm, Sandbox},
};

mod env;

lazy_static! {
    static ref MPC_INTENT_WASM: Vec<u8> = read_wasm("defuse-mpc-intent-contract");
}

pub trait MpcIntent {
    async fn deploy_mpc_intent_shard(&self, mpc_intent_shard_id: impl AsRef<str>) -> Contract;

    async fn create_intent(
        &self,
        account_shard_id: &AccountId,
        derivation_path: impl AsRef<str>,
        mpc_intent_id: &AccountId,
        id: IntentID,
        intent: Intent,
    );

    async fn fulfill_intent(
        &self,
        account_shard_id: &AccountId,
        derivation_path: impl AsRef<str>,
        mpc_intent_id: &AccountId,
        id: IntentID,
        recipient: impl Into<Option<AccountId>>,
    );
}

impl MpcIntent for near_workspaces::Account {
    async fn deploy_mpc_intent_shard(&self, mpc_intent_shard_id: impl AsRef<str>) -> Contract {
        let contract = self
            .deploy_contract(mpc_intent_shard_id, &MPC_INTENT_WASM)
            .await;

        contract
            .call("new")
            .max_gas()
            .transact()
            .await
            .unwrap()
            .into_result()
            .unwrap();

        contract
    }

    async fn create_intent(
        &self,
        account_shard_id: &AccountId,
        derivation_path: impl AsRef<str>,
        mpc_intent_id: &AccountId,
        id: IntentID,
        intent: Intent,
    ) {
        self.transfer_account_call(
            account_shard_id,
            derivation_path,
            mpc_intent_id,
            None,
            Action::Create { id, intent }.encode().unwrap(),
        )
        .await
    }

    async fn fulfill_intent(
        &self,
        account_shard_id: &AccountId,
        derivation_path: impl AsRef<str>,
        mpc_intent_id: &AccountId,
        id: IntentID,
        recipient: impl Into<Option<AccountId>>,
    ) {
        self.transfer_account_call(
            account_shard_id,
            derivation_path,
            mpc_intent_id,
            None,
            Action::Fulfill {
                id,
                recipient: recipient.into(),
                memo: None,
                msg: None,
            }
            .encode()
            .unwrap(),
        )
        .await
    }
}

#[tokio::test]
async fn test_mpc_intent() {
    let sandbox = Sandbox::new().await.unwrap();

    let dao = sandbox
        .create_subaccount("dao", NearToken::from_near(100))
        .await
        .unwrap();
    let account_shard = dao
        .deploy_account_shard(
            "account-shard",
            &"mpc.controller.near".parse().unwrap(),
            None,
        )
        .await;
    let mpc_intent = dao.deploy_mpc_intent_shard("mpc-intent").await;

    let user = sandbox.create_account("user").await;
    let solver = sandbox.create_account("solver").await;

    user.create_account(account_shard.id(), "a", None).await;
    solver.create_account(account_shard.id(), "b", None).await;

    let intent_id = "1".to_string();
    user.create_intent(
        account_shard.id(),
        "a",
        mpc_intent.id(),
        intent_id.clone(),
        Intent::Barter(BarterIntent {
            receive: Account {
                account_shard: account_shard.id().clone(),
                derivation_path: "b".to_string(),
            },
            recepient: user.id().clone(),
            memo: None,
            msg: None,
        }),
    )
    .await;

    assert_eq!(
        account_shard.owner_of("a").await,
        Some(mpc_intent.id().clone())
    );

    solver
        .fulfill_intent(account_shard.id(), "b", mpc_intent.id(), intent_id, None)
        .await;

    assert_eq!(account_shard.owner_of("a").await, Some(solver.id().clone()));
    assert_eq!(account_shard.owner_of("b").await, Some(user.id().clone()));
}
