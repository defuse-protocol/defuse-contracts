use defuse_intent_contract::types::IntentType;
use near_workspaces::types::NearToken;
use near_workspaces::{Account, AccountId, Contract};
use serde_json::json;

pub trait Intent {
    async fn create_intent(&self, contract_id: &AccountId, id: &str, intent: IntentType);

    async fn execute_intent(&self, contract_id: &AccountId, id: &str);

    async fn add_solver(&self, solver_id: &AccountId);
}

impl Intent for Account {
    async fn create_intent(&self, contract_id: &AccountId, id: &str, intent: IntentType) {
        let result = self
            .call(contract_id, "create_intent")
            .args_json(json!({
                "id": id,
                "intent": intent
            }))
            .deposit(NearToken::from_yoctonear(1))
            .transact()
            .await
            .unwrap();
        assert!(result.is_success(), "creation intent error: {result:#?}");
    }

    async fn execute_intent(&self, contract_id: &AccountId, id: &str) {
        let result = self
            .call(contract_id, "execute_intent")
            .args_json(json!({
                "intent_id": id
            }))
            .transact()
            .await
            .unwrap();
        assert!(result.is_success(), "execution intent error: {result:#?}");
    }

    async fn add_solver(&self, solver_id: &AccountId) {}
}

impl Intent for Contract {
    async fn create_intent(&self, contract_id: &AccountId, id: &str, intent: IntentType) {
        todo!()
    }

    async fn execute_intent(&self, contract_id: &AccountId, id: &str) {
        todo!()
    }

    async fn add_solver(&self, solver_id: &AccountId) {
        let result = self
            .call("add_solver")
            .args_json(json!({
                "solver_id": solver_id
            }))
            .transact()
            .await
            .unwrap();
        assert!(result.is_success(), "execution intent error: {result:#?}");
    }
}
