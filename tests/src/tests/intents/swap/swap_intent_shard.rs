use anyhow::anyhow;
use defuse_contracts::{
    intents::swap::{
        Asset, CrossChainAsset, FtAmount, IntentId, NearAsset, NftItem, SwapIntent,
        SwapIntentAction,
    },
    utils::Mutex,
};
use lazy_static::lazy_static;
use near_sdk::{AccountId, NearToken};
use near_workspaces::Contract;
use serde_json::json;

use crate::utils::{
    account::AccountExt, cross_chain::CrossChainReceiverExt, ft::FtExt, native::NativeReceiverExt,
    nft::NftExt, read_wasm,
};

lazy_static! {
    static ref SWAP_INTENT_WASM: Vec<u8> = read_wasm("defuse_swap_intent_contract");
}

pub trait SwapIntentShard {
    async fn deploy_swap_intent_shard(
        &self,
        swap_intent_shard_id: &str,
    ) -> anyhow::Result<Contract>;

    async fn swap_intent_action(
        &self,
        swap_intent_id: &AccountId,
        asset_in: Asset,
        action: SwapIntentAction,
    ) -> anyhow::Result<bool>;

    async fn get_intent(&self, id: &IntentId) -> anyhow::Result<Option<Mutex<SwapIntent>>>;

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

    async fn swap_intent_action(
        &self,
        swap_intent_id: &AccountId,
        asset_in: Asset,
        action: SwapIntentAction,
    ) -> anyhow::Result<bool> {
        match asset_in {
            Asset::Near(NearAsset::Native { amount }) => {
                self.native_on_transfer(swap_intent_id, amount, &serde_json::to_string(&action)?)
                    .await
            }
            Asset::Near(NearAsset::Nep141(FtAmount { token, amount })) => Ok(self
                .ft_transfer_call(
                    &token,
                    swap_intent_id,
                    amount.0,
                    None,
                    &serde_json::to_string(&action)?,
                )
                .await?
                == amount.0),
            Asset::Near(NearAsset::Nep171(NftItem {
                collection,
                token_id,
            })) => {
                self.nft_transfer_call(
                    &collection,
                    swap_intent_id,
                    token_id,
                    None,
                    serde_json::to_string(&action)?,
                )
                .await
            }
            Asset::CrossChain(CrossChainAsset {
                oracle,
                asset,
                amount,
            }) => {
                if self.id() != &oracle {
                    return Err(anyhow!(
                        "this cross-chain asset must be sent from oracle {oracle}"
                    ));
                }
                self.cross_chain_on_transfer(
                    swap_intent_id,
                    asset,
                    amount,
                    serde_json::to_string(&action)?,
                )
                .await
            }
        }
    }

    async fn get_intent(&self, id: &IntentId) -> anyhow::Result<Option<Mutex<SwapIntent>>> {
        self.view(self.id(), "get_intent")
            .args_json(json!({
                "id": id,
            }))
            .await?
            .json()
            .map_err(Into::into)
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

impl SwapIntentShard for Contract {
    async fn deploy_swap_intent_shard(&self, swap_intent_shard_id: &str) -> anyhow::Result<Self> {
        self.as_account()
            .deploy_swap_intent_shard(swap_intent_shard_id)
            .await
    }

    async fn swap_intent_action(
        &self,
        swap_intent_id: &AccountId,
        asset_in: Asset,
        action: SwapIntentAction,
    ) -> anyhow::Result<bool> {
        self.as_account()
            .swap_intent_action(swap_intent_id, asset_in, action)
            .await
    }

    async fn get_intent(&self, id: &IntentId) -> anyhow::Result<Option<Mutex<SwapIntent>>> {
        self.as_account().get_intent(id).await
    }

    async fn rollback_intent(
        &self,
        swap_intent_id: &AccountId,
        id: &IntentId,
    ) -> anyhow::Result<bool> {
        self.as_account().rollback_intent(swap_intent_id, id).await
    }

    async fn lost_found(&self, swap_intent_id: &AccountId, id: &IntentId) -> anyhow::Result<bool> {
        self.as_account().lost_found(swap_intent_id, id).await
    }
}
