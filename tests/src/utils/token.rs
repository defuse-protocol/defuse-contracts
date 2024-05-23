use near_workspaces::types::NearToken;
use near_workspaces::{Account, AccountId, Contract};
use serde_json::json;
use std::str::FromStr;

const STORAGE_DEPOSIT: NearToken = NearToken::from_yoctonear(2_350_000_000_000_000_000_000);

pub trait Token {
    async fn ft_balance_of(&self, account_id: &AccountId) -> u128;
    async fn ft_transfer(&self, receiver_id: &AccountId, amount: u128);
    async fn storage_deposit(&self, account_id: &AccountId);
    async fn register_accounts(&self, accounts: &[&Account]);
}

impl Token for Contract {
    async fn ft_balance_of(&self, account_id: &AccountId) -> u128 {
        let result = self
            .view("ft_balance_of")
            .args_json(json!({
                "account_id": account_id
            }))
            .await
            .unwrap();
        let value: String = result.json().unwrap();
        u128::from_str(&value).unwrap()
    }

    async fn ft_transfer(&self, receiver_id: &AccountId, amount: u128) {
        let result = self
            .call("ft_transfer")
            .args_json(json!({ "receiver_id": receiver_id, "amount": amount.to_string() }))
            .deposit(NearToken::from_yoctonear(1))
            .max_gas()
            .transact()
            .await
            .unwrap();
        assert!(result.is_success(), "{result:?}");
    }

    async fn storage_deposit(&self, account_id: &AccountId) {
        let result = self
            .call("storage_deposit")
            .args_json(json!({"account_id": account_id }))
            .deposit(STORAGE_DEPOSIT)
            .max_gas()
            .transact()
            .await
            .unwrap();
        assert!(result.is_success(), "{result:?}");
    }

    async fn register_accounts(&self, accounts: &[&Account]) {
        for account in accounts {
            self.storage_deposit(account.id()).await;
        }
    }
}
