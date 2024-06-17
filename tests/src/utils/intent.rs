use defuse_contracts::intent::{Action, DetailedIntent, Intent};

use near_sdk::json_types::U128;
use near_workspaces::{result::ExecutionFinalResult, types::NearToken, Account, AccountId};
use serde_json::json;

pub trait Intending {
    async fn create_intent(
        &self,
        contract_id: &AccountId,
        intent_account_id: &AccountId,
        id: &str,
        intent: Intent,
    );
    async fn execute_intent(
        &self,
        contract_id: &AccountId,
        intent_account_id: &AccountId,
        id: &str,
        amount: U128,
    );
    async fn rollback_intent(&self, contract_id: &AccountId, id: &str) -> ExecutionFinalResult;
    async fn add_solver(&self, contract_id: &AccountId, solver_id: &AccountId);
    async fn set_min_ttl(&self, contract_id: &AccountId, min_ttl: u64);

    // View transactions
    async fn get_intent(&self, intent_contract_id: &AccountId, id: &str) -> Option<DetailedIntent>;
}

impl Intending for Account {
    async fn create_intent(
        &self,
        contract_id: &AccountId,
        intent_account_id: &AccountId,
        id: &str,
        intent: Intent,
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
            .max_gas()
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
            .max_gas()
            .transact()
            .await
            .unwrap();
        assert!(result.is_success(), "execution intent error: {result:#?}");
    }

    async fn rollback_intent(&self, contract_id: &AccountId, id: &str) -> ExecutionFinalResult {
        self.call(contract_id, "rollback_intent")
            .args_json(json!({
                "id": id
            }))
            .max_gas()
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
