use defuse_contracts::{
    intents::swap::{
        Asset, CreateSwapIntentAction, FtAmount, FulfillSwapIntentAction, IntentId, NftItem,
        SwapIntent, SwapIntentAction,
    },
    utils::Mutex,
};
use lazy_static::lazy_static;
use near_sdk::AccountId;
use near_workspaces::Contract;
use serde_json::json;

use crate::utils::{account::AccountExt, ft::FtExt, nft::NftExt, read_wasm};

lazy_static! {
    static ref SWAP_INTENT_WASM: Vec<u8> = read_wasm("defuse-swap-intent-contract");
}

pub trait SwapIntentShard {
    async fn deploy_swap_intent_shard(&self, swap_intent_shard_id: impl AsRef<str>) -> Contract;

    async fn create_swap_intent(
        &self,
        swap_intent_id: &AccountId,
        asset_in: Asset,
        create: CreateSwapIntentAction,
    );

    async fn get_swap_intent(&self, id: &IntentId) -> Option<Mutex<SwapIntent>>;

    async fn fulfill_swap_intent(
        &self,
        swap_intent_id: &AccountId,
        asset_in: Asset,
        fulfill: FulfillSwapIntentAction,
    );

    async fn rollback_intent(&self, swap_intent_id: &AccountId, id: &IntentId) -> bool;

    async fn lost_found(&self, swap_intent_id: &AccountId, id: &IntentId) -> bool;
}

impl SwapIntentShard for near_workspaces::Account {
    async fn deploy_swap_intent_shard(&self, swap_intent_shard_id: impl AsRef<str>) -> Contract {
        let contract = self
            .deploy_contract(swap_intent_shard_id, &SWAP_INTENT_WASM)
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

    async fn create_swap_intent(
        &self,
        swap_intent_id: &AccountId,
        asset_in: Asset,
        create: CreateSwapIntentAction,
    ) {
        match asset_in {
            Asset::Native(amount) => {
                self.call(swap_intent_id, "native_action")
                    .args_json(json!({
                        "action": SwapIntentAction::Create(create),
                    }))
                    .deposit(amount)
                    .max_gas()
                    .transact()
                    .await
                    .unwrap()
                    .into_result()
                    .unwrap();
            }
            Asset::Ft(FtAmount { token, amount }) => {
                assert_eq!(
                    self.ft_transfer_call(
                        &token,
                        swap_intent_id,
                        amount,
                        None,
                        serde_json::to_string(&SwapIntentAction::Create(create)).unwrap(),
                    )
                    .await,
                    amount,
                );
            }
            Asset::Nft(NftItem {
                collection,
                token_id,
            }) => assert!(
                self.nft_transfer_call(
                    &collection,
                    swap_intent_id,
                    token_id,
                    None,
                    serde_json::to_string(&SwapIntentAction::Create(create)).unwrap()
                )
                .await
            ),
        }
    }

    async fn get_swap_intent(&self, id: &IntentId) -> Option<Mutex<SwapIntent>> {
        self.view(self.id(), "get_swap_intent")
            .args_json(json!({
                "id": id,
            }))
            .await
            .unwrap()
            .json()
            .unwrap()
    }

    async fn fulfill_swap_intent(
        &self,
        swap_intent_id: &AccountId,
        asset_in: Asset,
        fulfill: FulfillSwapIntentAction,
    ) {
        match asset_in {
            Asset::Native(amount) => {
                self.call(swap_intent_id, "native_action")
                    .args_json(json!({
                        "action": SwapIntentAction::Fulfill(fulfill),
                    }))
                    .deposit(amount)
                    .max_gas()
                    .transact()
                    .await
                    .unwrap()
                    .into_result()
                    .unwrap();
            }
            Asset::Ft(FtAmount { token, amount }) => {
                assert_eq!(
                    self.ft_transfer_call(
                        &token,
                        swap_intent_id,
                        amount,
                        None,
                        serde_json::to_string(&SwapIntentAction::Fulfill(fulfill)).unwrap(),
                    )
                    .await,
                    amount,
                );
            }
            Asset::Nft(NftItem {
                collection,
                token_id,
            }) => assert!(
                self.nft_transfer_call(
                    &collection,
                    swap_intent_id,
                    token_id,
                    None,
                    serde_json::to_string(&SwapIntentAction::Fulfill(fulfill)).unwrap(),
                )
                .await
            ),
        }
    }

    async fn rollback_intent(&self, swap_intent_id: &AccountId, id: &IntentId) -> bool {
        self.call(swap_intent_id, "rollback_intent")
            .args_json(json!({
                "id": id,
            }))
            .max_gas()
            .transact()
            .await
            .unwrap()
            .into_result()
            .unwrap()
            .json()
            .unwrap()
    }

    async fn lost_found(&self, swap_intent_id: &AccountId, id: &IntentId) -> bool {
        self.call(swap_intent_id, "lost_found")
            .args_json(json!({
                "id": id,
            }))
            .max_gas()
            .transact()
            .await
            .unwrap()
            .into_result()
            .unwrap()
            .json()
            .unwrap()
    }
}