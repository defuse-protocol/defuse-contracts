use defuse_contracts::intents::swap_ft::{Action, DetailedIntent, Intent};
use lazy_static::lazy_static;
use near_sdk::{json_types::U128, AccountId, Gas, NearToken};
use near_workspaces::{
    operations::TransactionStatus, result::ExecutionFinalResult, Account, Contract,
};
use serde_json::json;

use crate::utils::{account::AccountExt, read_wasm};

lazy_static! {
    static ref SWAP_FT_INTENT_WASM: Vec<u8> = read_wasm("defuse-swap-ft-intent-contract");
}

pub trait SwapFtIntentExt {
    async fn deploy_swap_ft_intent_contract(&self) -> anyhow::Result<Contract>;

    async fn create_intent(
        &self,
        contract_id: &AccountId,
        intent_account_id: &AccountId,
        id: &str,
        intent: Intent,
    );
    async fn create_intent_with_gas(
        &self,
        contract_id: &AccountId,
        intent_account_id: &AccountId,
        id: &str,
        intent: Intent,
        gas: Gas,
    );
    async fn execute_intent(
        &self,
        contract_id: &AccountId,
        intent_account_id: &AccountId,
        id: &str,
        amount: U128,
    );
    async fn execute_intent_async(
        &self,
        contract_id: &AccountId,
        intent_account_id: &AccountId,
        id: &str,
        amount: U128,
    ) -> TransactionStatus;
    async fn execute_intent_with_gas(
        &self,
        contract_id: &AccountId,
        intent_account_id: &AccountId,
        id: &str,
        amount: U128,
        gas: Gas,
    );
    async fn rollback_intent(&self, contract_id: &AccountId, id: &str) -> ExecutionFinalResult;
    async fn add_solver(&self, contract_id: &AccountId, solver_id: &AccountId);
    async fn set_min_ttl(&self, contract_id: &AccountId, min_ttl: u64);

    // View transactions
    async fn get_intent(&self, intent_contract_id: &AccountId, id: &str) -> Option<DetailedIntent>;
}

impl SwapFtIntentExt for Account {
    async fn deploy_swap_ft_intent_contract(&self) -> anyhow::Result<Contract> {
        let contract = self.deploy_contract("intent", &SWAP_FT_INTENT_WASM).await?;
        contract
            .call("new")
            .args_json(json!({
                "owner_id": contract.id()
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?;

        Ok(contract)
    }
    async fn create_intent(
        &self,
        contract_id: &AccountId,
        intent_account_id: &AccountId,
        id: &str,
        intent: Intent,
    ) {
        self.create_intent_with_gas(
            contract_id,
            intent_account_id,
            id,
            intent,
            Gas::from_tgas(50),
        )
        .await;
    }

    async fn create_intent_with_gas(
        &self,
        contract_id: &AccountId,
        intent_account_id: &AccountId,
        id: &str,
        intent: Intent,
        gas: Gas,
    ) {
        let amount = intent.send.amount;
        let intent = Action::CreateIntent(id.to_string(), intent);
        let msg = intent.encode().expect("encode Action");
        let args = json!({
            "receiver_id": intent_account_id,
            "amount": amount,
            "memo": "Create intent: NEP-141 to NEP-141",
            "msg": msg
        });

        let result = self
            .call(contract_id, "ft_transfer_call")
            .args_json(args)
            .deposit(NearToken::from_yoctonear(1))
            .gas(gas)
            .transact()
            .await
            .unwrap();
        assert!(result.is_success(), "creation intent error: {result:#?}");
    }

    async fn execute_intent(
        &self,
        contract_id: &AccountId,
        intent_account_id: &AccountId,
        id: &str,
        amount: U128,
    ) {
        self.execute_intent_with_gas(
            contract_id,
            intent_account_id,
            id,
            amount,
            Gas::from_tgas(65),
        )
        .await;
    }

    async fn execute_intent_with_gas(
        &self,
        contract_id: &AccountId,
        intent_account_id: &AccountId,
        id: &str,
        amount: U128,
        gas: Gas,
    ) {
        let intent = Action::ExecuteIntent(id.to_string());
        let msg = intent.encode().expect("encode Action");
        let args = json!({
            "receiver_id": intent_account_id,
            "amount": amount,
            "memo": "Execute intent: NEP-141 to NEP-141",
            "msg": msg
        });

        let result = self
            .call(contract_id, "ft_transfer_call")
            .args_json(args)
            .deposit(NearToken::from_yoctonear(1))
            .gas(gas)
            .transact()
            .await
            .unwrap();
        assert!(result.is_success(), "execution intent error: {result:#?}");
    }

    async fn execute_intent_async(
        &self,
        contract_id: &AccountId,
        intent_account_id: &AccountId,
        id: &str,
        amount: U128,
    ) -> TransactionStatus {
        let intent = Action::ExecuteIntent(id.to_string());
        let msg = intent.encode().expect("encode Action");
        let args = json!({
            "receiver_id": intent_account_id,
            "amount": amount,
            "memo": "Execute intent: NEP-141 to NEP-141",
            "msg": msg
        });

        self.call(contract_id, "ft_transfer_call")
            .args_json(args)
            .deposit(NearToken::from_yoctonear(1))
            .max_gas()
            .transact_async()
            .await
            .unwrap()
    }

    async fn rollback_intent(&self, contract_id: &AccountId, id: &str) -> ExecutionFinalResult {
        self.call(contract_id, "rollback_intent")
            .args_json(json!({
                "id": id
            }))
            .gas(Gas::from_tgas(20))
            .transact()
            .await
            .unwrap()
    }

    async fn add_solver(&self, contract_id: &AccountId, solver_id: &AccountId) {
        let result = self
            .call(contract_id, "add_solver")
            .args_json(json!({
                "solver_id": solver_id
            }))
            .transact()
            .await
            .unwrap();
        assert!(result.is_success(), "execution intent error: {result:#?}");
    }

    async fn set_min_ttl(&self, contract_id: &AccountId, min_ttl: u64) {
        let result = self
            .call(contract_id, "set_min_intent_ttl")
            .args_json(json!({
                "min_ttl": min_ttl
            }))
            .transact()
            .await
            .unwrap();
        assert!(result.is_success(), "set min ttl error: {result:#?}");
    }

    // View transactions
    async fn get_intent(&self, intent_contract_id: &AccountId, id: &str) -> Option<DetailedIntent> {
        let result = self
            .call(intent_contract_id, "get_intent")
            .args_json(json!({
                "id": id
            }))
            .view()
            .await
            .unwrap();

        result.json().unwrap()
    }
}

impl SwapFtIntentExt for Contract {
    async fn deploy_swap_ft_intent_contract(&self) -> anyhow::Result<Self> {
        self.as_account().deploy_swap_ft_intent_contract().await
    }

    async fn create_intent(
        &self,
        contract_id: &AccountId,
        intent_account_id: &AccountId,
        id: &str,
        intent: Intent,
    ) {
        self.as_account()
            .create_intent(contract_id, intent_account_id, id, intent)
            .await;
    }

    async fn create_intent_with_gas(
        &self,
        contract_id: &AccountId,
        intent_account_id: &AccountId,
        id: &str,
        intent: Intent,
        gas: Gas,
    ) {
        self.as_account()
            .create_intent_with_gas(contract_id, intent_account_id, id, intent, gas)
            .await;
    }

    async fn execute_intent(
        &self,
        contract_id: &AccountId,
        intent_account_id: &AccountId,
        id: &str,
        amount: U128,
    ) {
        self.as_account()
            .execute_intent(contract_id, intent_account_id, id, amount)
            .await;
    }

    async fn execute_intent_async(
        &self,
        contract_id: &AccountId,
        intent_account_id: &AccountId,
        id: &str,
        amount: U128,
    ) -> TransactionStatus {
        self.as_account()
            .execute_intent_async(contract_id, intent_account_id, id, amount)
            .await
    }

    async fn execute_intent_with_gas(
        &self,
        contract_id: &AccountId,
        intent_account_id: &AccountId,
        id: &str,
        amount: U128,
        gas: Gas,
    ) {
        self.as_account()
            .execute_intent_with_gas(contract_id, intent_account_id, id, amount, gas)
            .await;
    }

    async fn rollback_intent(&self, contract_id: &AccountId, id: &str) -> ExecutionFinalResult {
        self.as_account().rollback_intent(contract_id, id).await
    }

    async fn add_solver(&self, contract_id: &AccountId, solver_id: &AccountId) {
        self.as_account().add_solver(contract_id, solver_id).await;
    }

    async fn set_min_ttl(&self, contract_id: &AccountId, min_ttl: u64) {
        self.as_account().set_min_ttl(contract_id, min_ttl).await;
    }

    async fn get_intent(&self, intent_contract_id: &AccountId, id: &str) -> Option<DetailedIntent> {
        self.as_account().get_intent(intent_contract_id, id).await
    }
}
