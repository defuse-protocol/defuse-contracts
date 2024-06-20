use lazy_static::lazy_static;
use near_workspaces::Contract;
use serde_json::json;

use crate::utils::{account::AccountExt, read_wasm};

mod env;

lazy_static! {
    static ref MPC_INTENT_WASM: Vec<u8> = read_wasm("defuse-mpc-intent-contract");
}

pub trait MpcIntent {
    async fn deploy_mpc_intent_shard(&self, mpc_intent_shard_id: impl AsRef<str>) -> Contract;

    async fn create_intent(&self);
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

    async fn create_intent(&self) {
        todo!()
    }
}
