pub mod accounts;
mod env;
mod intents;
mod tokens;

use std::sync::LazyLock;

use defuse_contract::config::DefuseConfig;
use defuse_contracts::{
    crypto::SignedPayload,
    defuse::payload::{multi::MultiStandardPayload, nep413::Nep413DefuseMessage},
    nep413::{Nep413Payload, U256},
    utils::Deadline,
};
use near_sdk::{serde::Serialize, serde_json::json, AccountId};
use near_workspaces::Contract;

use crate::utils::{account::AccountExt, crypto::Signer, read_wasm};

use self::accounts::AccountManagerExt;

static DEFUSE_WASM: LazyLock<Vec<u8>> = LazyLock::new(|| read_wasm("defuse_contract"));

pub trait DefuseExt: AccountManagerExt {
    #[allow(clippy::too_many_arguments)]
    async fn deploy_defuse(&self, id: &str, config: DefuseConfig) -> anyhow::Result<Contract>;
}

impl DefuseExt for near_workspaces::Account {
    async fn deploy_defuse(&self, id: &str, config: DefuseConfig) -> anyhow::Result<Contract> {
        let contract = self.deploy_contract(id, &DEFUSE_WASM).await?;
        contract
            .call("new")
            .args_json(json!({
                "config": config,
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?;
        Ok(contract)
    }
}

impl DefuseExt for Contract {
    async fn deploy_defuse(&self, id: &str, config: DefuseConfig) -> anyhow::Result<Contract> {
        self.as_account().deploy_defuse(id, config).await
    }
}

pub trait DefuseSigner: Signer {
    fn sign_defuse_message<T>(
        &self,
        defuse_contract: &AccountId,
        nonce: U256,
        deadline: Deadline,
        message: T,
    ) -> SignedPayload<MultiStandardPayload>
    where
        T: Serialize;
}

impl DefuseSigner for near_workspaces::Account {
    fn sign_defuse_message<T>(
        &self,
        defuse_contract: &AccountId,
        nonce: U256,
        deadline: Deadline,
        message: T,
    ) -> SignedPayload<MultiStandardPayload>
    where
        T: Serialize,
    {
        self.sign_payload(MultiStandardPayload::Nep413(
            Nep413Payload::new(
                serde_json::to_string(&Nep413DefuseMessage {
                    signer_id: self.id().clone(),
                    deadline,
                    message,
                })
                .unwrap(),
            )
            .with_recipient(defuse_contract)
            .with_nonce(nonce),
        ))
    }
}
