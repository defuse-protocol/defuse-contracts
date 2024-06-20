use lazy_static::lazy_static;
use near_contract_standards::non_fungible_token::Token;
use near_sdk::{AccountId, NearToken};
use near_workspaces::Contract;
use serde_json::json;

use crate::utils::{account::AccountExt, read_wasm, Sandbox};

lazy_static! {
    static ref ACCOUNT_WASM: Vec<u8> = read_wasm("defuse-account-contract");
}

pub trait Account {
    async fn deploy_account_shard(
        &self,
        account_shard_id: impl AsRef<str>,
        mpc_contract_id: &AccountId,
        owner: impl Into<Option<AccountId>>,
    ) -> Contract;

    async fn create_account(
        &self,
        account_shard_id: &AccountId,
        derivation_path: impl AsRef<str>,
        owner: impl Into<Option<AccountId>>,
    );

    async fn transfer_account(
        &self,
        account_shard_id: &AccountId,
        receiver_id: &AccountId,
        derivation_path: impl AsRef<str>,
        memo: impl Into<Option<String>>,
    );

    async fn transfer_account_call(
        &self,
        account_shard_id: &AccountId,
        receiver_id: &AccountId,
        derivation_path: impl AsRef<str>,
        memo: impl Into<Option<String>>,
        msg: String,
    );
}
impl Account for near_workspaces::Account {
    async fn deploy_account_shard(
        &self,
        account_shard_id: impl AsRef<str>,
        mpc_contract_id: &AccountId,
        owner: impl Into<Option<AccountId>>,
    ) -> Contract {
        let contract = self.deploy_contract(account_shard_id, &ACCOUNT_WASM).await;

        contract
            .call("new")
            .args_json(json!({
                "owner": owner.into().unwrap_or(self.id().clone()),
                "mpc_contract_id": mpc_contract_id,
            }))
            .max_gas()
            .transact()
            .await
            .unwrap()
            .into_result()
            .unwrap();

        contract
    }

    async fn create_account(
        &self,
        account_shard_id: &AccountId,
        derivation_path: impl AsRef<str>,
        owner: impl Into<Option<AccountId>>,
    ) {
        self.call(account_shard_id, "create_account")
            .args_json(json!({
                "owner": owner.into().unwrap_or(self.id().clone()),
                "derivation_path": derivation_path.as_ref(),
            }))
            .deposit(NearToken::from_millinear(2))
            .max_gas()
            .transact()
            .await
            .unwrap()
            .into_result()
            .unwrap();
    }

    async fn transfer_account(
        &self,
        account_shard_id: &AccountId,
        receiver_id: &AccountId,
        derivation_path: impl AsRef<str>,
        memo: impl Into<Option<String>>,
    ) {
        self.call(account_shard_id, "nft_transfer")
            .args_json(json!({
                "receiver_id": receiver_id,
                "token_id": derivation_path.as_ref(),
                "approval_id": null,
                "memo": memo.into(),
            }))
            .deposit(NearToken::from_yoctonear(1))
            .max_gas()
            .transact()
            .await
            .unwrap()
            .into_result()
            .unwrap();
    }

    async fn transfer_account_call(
        &self,
        account_shard_id: &AccountId,
        receiver_id: &AccountId,
        derivation_path: impl AsRef<str>,
        memo: impl Into<Option<String>>,
        msg: String,
    ) {
        let success: bool = self
            .call(account_shard_id, "nft_transfer_call")
            .args_json(json!({
                "receiver_id": receiver_id,
                "token_id": derivation_path.as_ref(),
                "approval_id": null,
                "memo": memo.into(),
                "msg": msg,
            }))
            .deposit(NearToken::from_yoctonear(1))
            .max_gas()
            .transact()
            .await
            .unwrap()
            .into_result()
            .unwrap()
            .json()
            .unwrap();
        assert!(success);
    }
}

pub trait AccountContract {
    async fn owner_of(&self, derivation_path: impl AsRef<str>) -> Option<Token>;
}

impl AccountContract for Contract {
    async fn owner_of(&self, derivation_path: impl AsRef<str>) -> Option<Token> {
        self.view("nft_token")
            .args_json(json!({
                "token_id": derivation_path.as_ref(),
            }))
            .await
            .unwrap()
            .json()
            .unwrap()
    }
}

#[tokio::test]
async fn test_account_shard() {
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

    let user1 = sandbox.create_account("user1").await;
    let user2 = sandbox.create_account("user2").await;

    user1.create_account(account_shard.id(), "a", None).await;
    assert_eq!(
        account_shard
            .owner_of("a")
            .await
            .map(|token| token.owner_id),
        Some(user1.id().clone())
    );

    user1
        .transfer_account(account_shard.id(), user2.id(), "a", None)
        .await;
    assert_eq!(
        account_shard
            .owner_of("a")
            .await
            .map(|token| token.owner_id),
        Some(user2.id().clone())
    );
}
