use defuse_contracts::{
    intents::swap::{
        Asset, CreateSwapIntentAction, ExecuteSwapIntentAction, FtAmount, IntentId, NftItem,
        SwapIntentAction, SwapIntentStatus,
    },
    utils::Mutex,
};
use lazy_static::lazy_static;
use near_sdk::{AccountId, NearToken};
use near_workspaces::Contract;
use serde_json::json;

use crate::utils::{account::AccountExt, ft::FtExt, nft::NftExt, read_wasm};

lazy_static! {
    static ref SWAP_INTENT_WASM: Vec<u8> = read_wasm("defuse-swap-intent-contract");
}

pub trait SwapIntentShard {
    async fn deploy_swap_intent_shard(
        &self,
        swap_intent_shard_id: &str,
    ) -> anyhow::Result<Contract>;

    async fn create_swap_intent(
        &self,
        swap_intent_id: &AccountId,
        asset_in: Asset,
        create: CreateSwapIntentAction,
    ) -> anyhow::Result<bool>;

    async fn get_swap_intent(
        &self,
        id: &IntentId,
    ) -> anyhow::Result<Option<Mutex<SwapIntentStatus>>>;

    async fn fulfill_swap_intent(
        &self,
        swap_intent_id: &AccountId,
        asset_in: Asset,
        fulfill: ExecuteSwapIntentAction,
    ) -> anyhow::Result<bool>;

    async fn rollback_intent(
        &self,
        swap_intent_id: &AccountId,
        id: &IntentId,
    ) -> anyhow::Result<bool>;

    async fn lost_found(&self, swap_intent_id: &AccountId, id: &IntentId) -> anyhow::Result<bool>;
}

impl SwapIntentShard for near_workspaces::Account {
    async fn deploy_swap_intent_shard(
        &self,
        swap_intent_shard_id: &str,
    ) -> anyhow::Result<Contract> {
        let contract = self
            .deploy_contract(swap_intent_shard_id, &SWAP_INTENT_WASM)
            .await?;

        contract
            .call("new")
            .max_gas()
            .transact()
            .await?
            .into_result()?;

        Ok(contract)
    }

    async fn create_swap_intent(
        &self,
        swap_intent_id: &AccountId,
        asset_in: Asset,
        create: CreateSwapIntentAction,
    ) -> anyhow::Result<bool> {
        match asset_in {
            Asset::Native(amount) => self
                .call(swap_intent_id, "native_action")
                .args_json(json!({
                    "action": SwapIntentAction::Create(create),
                }))
                .deposit(amount)
                .max_gas()
                .transact()
                .await?
                .into_result()?
                .json()
                .map_err(Into::into),
            Asset::Ft(FtAmount { token, amount }) => Ok(self
                .ft_transfer_call(
                    &token,
                    swap_intent_id,
                    amount,
                    None,
                    &serde_json::to_string(&SwapIntentAction::Create(create)).unwrap(),
                )
                .await?
                == amount),
            Asset::Nft(NftItem {
                collection,
                token_id,
            }) => {
                self.nft_transfer_call(
                    &collection,
                    swap_intent_id,
                    token_id,
                    None,
                    serde_json::to_string(&SwapIntentAction::Create(create)).unwrap(),
                )
                .await
            }
        }
    }

    async fn get_swap_intent(
        &self,
        id: &IntentId,
    ) -> anyhow::Result<Option<Mutex<SwapIntentStatus>>> {
        self.view(self.id(), "get_swap_intent")
            .args_json(json!({
                "id": id,
            }))
            .await?
            .json()
            .map_err(Into::into)
    }

    async fn fulfill_swap_intent(
        &self,
        swap_intent_id: &AccountId,
        asset_in: Asset,
        fulfill: ExecuteSwapIntentAction,
    ) -> anyhow::Result<bool> {
        match asset_in {
            Asset::Native(amount) => self
                .call(swap_intent_id, "native_action")
                .args_json(json!({
                    "action": SwapIntentAction::Execute(fulfill),
                }))
                .deposit(amount)
                .max_gas()
                .transact()
                .await?
                .into_result()?
                .json()
                .map_err(Into::into),
            Asset::Ft(FtAmount { token, amount }) => Ok(self
                .ft_transfer_call(
                    &token,
                    swap_intent_id,
                    amount,
                    None,
                    &serde_json::to_string(&SwapIntentAction::Execute(fulfill)).unwrap(),
                )
                .await?
                == amount),
            Asset::Nft(NftItem {
                collection,
                token_id,
            }) => {
                self.nft_transfer_call(
                    &collection,
                    swap_intent_id,
                    token_id,
                    None,
                    serde_json::to_string(&SwapIntentAction::Execute(fulfill)).unwrap(),
                )
                .await
            }
        }
    }

    async fn rollback_intent(
        &self,
        swap_intent_id: &AccountId,
        id: &IntentId,
    ) -> anyhow::Result<bool> {
        self.call(swap_intent_id, "rollback_intent")
            .args_json(json!({
                "id": id,
            }))
            .deposit(NearToken::from_yoctonear(1))
            .max_gas()
            .transact()
            .await?
            .into_result()?
            .json()
            .map_err(Into::into)
    }

    async fn lost_found(&self, swap_intent_id: &AccountId, id: &IntentId) -> anyhow::Result<bool> {
        self.call(swap_intent_id, "lost_found")
            .args_json(json!({
                "id": id,
            }))
            .deposit(NearToken::from_yoctonear(1))
            .max_gas()
            .transact()
            .await?
            .into_result()?
            .json()
            .map_err(Into::into)
    }
}
