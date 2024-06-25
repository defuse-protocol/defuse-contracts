use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::{AccountId, NearToken};
use serde_json::json;

pub trait NftExt {
    async fn nft_transfer(
        &self,
        collection: &AccountId,
        receiver_id: &AccountId,
        token_id: TokenId,
        memo: impl Into<Option<String>>,
    );

    async fn nft_transfer_call(
        &self,
        collection: &AccountId,
        receiver_id: &AccountId,
        token_id: TokenId,
        memo: impl Into<Option<String>>,
        msg: String,
    ) -> bool;

    async fn nft_token(&self, token_id: &TokenId) -> Option<Token>;
}

impl NftExt for near_workspaces::Account {
    async fn nft_transfer(
        &self,
        collection: &AccountId,
        receiver_id: &AccountId,
        token_id: TokenId,
        memo: impl Into<Option<String>>,
    ) {
        self.call(&collection, "nft_transfer")
            .args_json(json!({
                "receiver_id": receiver_id,
                "token_id": token_id,
                "memo": memo.into()
            }))
            .deposit(NearToken::from_yoctonear(1))
            .max_gas()
            .transact()
            .await
            .unwrap()
            .into_result()
            .unwrap();
    }

    async fn nft_transfer_call(
        &self,
        collection: &AccountId,
        receiver_id: &AccountId,
        token_id: TokenId,
        memo: impl Into<Option<String>>,
        msg: String,
    ) -> bool {
        self.call(&collection, "nft_transfer_call")
            .args_json(json!({
                "receiver_id": receiver_id,
                "token_id": token_id,
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
            .unwrap()
    }

    async fn nft_token(&self, token_id: &TokenId) -> Option<Token> {
        self.view(self.id(), "nft_token")
            .args_json(json!({
                "token_id": token_id,
            }))
            .await
            .unwrap()
            .json()
            .unwrap()
    }
}
