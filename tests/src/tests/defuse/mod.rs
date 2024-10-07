pub mod accounts;
mod env;
pub mod token_diff;
mod tokens;

use std::sync::LazyLock;

use accounts::AccountManagerExt;
use defuse_contracts::{
    defuse::{
        fees::Fees,
        message::{DefuseMessage, SignedDefuseMessage},
        payload::MultiStandardPayload,
    },
    nep413::{Nep413Payload, Nonce},
    utils::Deadline,
};
use near_sdk::{borsh::BorshSerialize, AccountId};
use near_workspaces::Contract;
use serde_json::json;

use crate::utils::{account::AccountExt, crypto::Signer, read_wasm};

static DEFUSE_WASM: LazyLock<Vec<u8>> = LazyLock::new(|| read_wasm("defuse_contract"));

pub trait DefuseExt: AccountManagerExt {
    async fn deploy_defuse(&self, id: &str, fees: Fees) -> anyhow::Result<Contract>;
}

impl DefuseExt for near_workspaces::Account {
    async fn deploy_defuse(&self, id: &str, fees: Fees) -> anyhow::Result<Contract> {
        let contract = self.deploy_contract(id, &DEFUSE_WASM).await?;

        contract
            .call("new")
            .args_json(json!({
                "fees": fees,
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?;

        Ok(contract)
    }
}

impl DefuseExt for Contract {
    async fn deploy_defuse(&self, id: &str, fees: Fees) -> anyhow::Result<Contract> {
        self.as_account().deploy_defuse(id, fees).await
    }
}

pub trait DefuseSigner: Signer {
    fn sign_defuse_message<T>(
        &self,
        defuse_contract: &AccountId,
        message: T,
        nonce: Nonce,
        deadline: Deadline,
    ) -> SignedDefuseMessage<T>
    where
        T: BorshSerialize;
}

impl DefuseSigner for near_workspaces::Account {
    fn sign_defuse_message<T>(
        &self,
        defuse_contract: &AccountId,
        message: T,
        nonce: Nonce,
        deadline: Deadline,
    ) -> SignedDefuseMessage<T>
    where
        T: BorshSerialize,
    {
        self.sign_payload(MultiStandardPayload::Nep413(
            Nep413Payload::new(DefuseMessage {
                signer_id: self.id().clone(),
                deadline,
                message,
            })
            .with_recipient(defuse_contract.to_string())
            .with_nonce(nonce),
        ))
    }
}
