use lazy_static::lazy_static;
use near_sdk::{AccountId, NearToken};
use near_workspaces::Contract;
use serde_json::json;

use crate::utils::{account::AccountExt, nft::NftExt, read_wasm, Sandbox};

lazy_static! {
    static ref ACCOUNT_WASM: Vec<u8> = read_wasm("defuse-account-contract");
}

pub trait AccountShardExt {
    async fn deploy_account_shard(
        &self,
        account_shard_id: impl AsRef<str>,
        owner: impl Into<Option<AccountId>>,
    ) -> Contract;

    async fn create_account(
        &self,
        account_shard_id: &AccountId,
        derivation_path: impl AsRef<str>,
        owner: impl Into<Option<AccountId>>,
    );
}

impl AccountShardExt for near_workspaces::Account {
    async fn deploy_account_shard(
        &self,
        account_shard_id: impl AsRef<str>,
        owner: impl Into<Option<AccountId>>,
    ) -> Contract {
        let contract = self.deploy_contract(account_shard_id, &ACCOUNT_WASM).await;

        contract
            .call("new")
            .args_json(json!({
                "owner": owner.into().unwrap_or(self.id().clone()),
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
                "derivation_path": derivation_path.as_ref(),
                "owner": owner.into(),
            }))
            .deposit(NearToken::from_millinear(2))
            .max_gas()
            .transact()
            .await
            .unwrap()
            .into_result()
            .unwrap();
    }
}

#[tokio::test]
async fn test_account_shard() {
    let sandbox = Sandbox::new().await.unwrap();

    let dao = sandbox
        .create_subaccount("dao", NearToken::from_near(100))
        .await
        .unwrap();
    let account_shard = dao.deploy_account_shard("account-shard", None).await;

    let user1 = sandbox.create_account("user1").await;
    let user2 = sandbox.create_account("user2").await;

    user1.create_account(account_shard.id(), "a", None).await;
    assert_eq!(
        &account_shard
            .as_account()
            .nft_token(&"a".to_string())
            .await
            .unwrap()
            .owner_id,
        user1.id()
    );

    user1
        .nft_transfer(account_shard.id(), user2.id(), "a".to_string(), None)
        .await;
    assert_eq!(
        &account_shard
            .as_account()
            .nft_token(&"a".to_string())
            .await
            .unwrap()
            .owner_id,
        user2.id()
    );
}
