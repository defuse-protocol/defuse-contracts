use lazy_static::lazy_static;
use near_contract_standards::storage_management::StorageBalance;
use near_sdk::json_types::U128;
use near_workspaces::types::NearToken;
use near_workspaces::{Account, AccountId, Contract};
use serde_json::json;

use crate::utils::read_wasm;

use super::account::AccountExt;
use super::storage_management::StorageManagementExt;

const STORAGE_DEPOSIT: NearToken = NearToken::from_yoctonear(2_350_000_000_000_000_000_000);
const TOTAL_SUPPLY: u128 = 1_000_000_000;

lazy_static! {
    static ref FUNGIBLE_TOKEN_WASM: Vec<u8> = read_wasm("fungible-token");
}

pub trait FtExt: StorageManagementExt {
    async fn deploy_ft_token(&self, token: impl AsRef<str>) -> anyhow::Result<Contract>;
    async fn ft_balance_of(&self, token_id: &AccountId) -> anyhow::Result<u128>;
    async fn ft_transfer(
        &self,
        token_id: &AccountId,
        receiver_id: &AccountId,
        amount: u128,
        memo: impl Into<Option<String>>,
    ) -> anyhow::Result<()>;
    async fn ft_transfer_call(
        &self,
        token_id: &AccountId,
        receiver_id: &AccountId,
        amount: u128,
        memo: impl Into<Option<String>>,
        msg: impl AsRef<str>,
    ) -> anyhow::Result<u128>;
    async fn ft_storage_deposit(
        &self,
        token_id: &AccountId,
        account_id: impl Into<Option<AccountId>>,
    ) -> anyhow::Result<StorageBalance> {
        self.storage_deposit(token_id, account_id, STORAGE_DEPOSIT)
            .await
    }
}

impl FtExt for Account {
    async fn deploy_ft_token(&self, token: impl AsRef<str>) -> anyhow::Result<Contract> {
        let token = token.as_ref();

        let contract = self.deploy_contract(token, &FUNGIBLE_TOKEN_WASM).await?;
        contract
            .call("new")
            .args_json(json!({
                "owner_id": contract.id(),
                    "total_supply": TOTAL_SUPPLY.to_string(),
                    "metadata": {
                        "spec": "ft-1.0.0",
                        "name": format!("Token {}", token),
                        "symbol": "TKN",
                        "decimals": 18
                    }
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?;

        Ok(contract)
    }

    async fn ft_balance_of(&self, account_id: &AccountId) -> anyhow::Result<u128> {
        self.view(self.id(), "ft_balance_of")
            .args_json(json!({
                "account_id": account_id,
            }))
            .await?
            .json::<String>()?
            .parse()
            .map_err(Into::into)
    }

    async fn ft_transfer(
        &self,
        token_id: &AccountId,
        receiver_id: &AccountId,
        amount: u128,
        memo: impl Into<Option<String>>,
    ) -> anyhow::Result<()> {
        self.call(token_id, "ft_transfer")
            .args_json(json!({
                "receiver_id": receiver_id,
                "amount": U128(amount),
                "memo": memo.into(),
            }))
            .deposit(NearToken::from_yoctonear(1))
            .max_gas()
            .transact()
            .await?
            .into_result()?;
        Ok(())
    }

    async fn ft_transfer_call(
        &self,
        token_id: &AccountId,
        receiver_id: &AccountId,
        amount: u128,
        memo: impl Into<Option<String>>,
        msg: impl AsRef<str>,
    ) -> anyhow::Result<u128> {
        self.call(token_id, "ft_transfer_call")
            .args_json(json!({
                "receiver_id": receiver_id,
                "amount": U128(amount),
                "memo": memo.into(),
                "msg": msg.as_ref(),
            }))
            .deposit(NearToken::from_yoctonear(1))
            .max_gas()
            .transact()
            .await?
            .into_result()?
            .json::<String>()?
            .parse()
            .map_err(Into::into)
    }
}
