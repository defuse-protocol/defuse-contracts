use near_workspaces::types::NearToken;
use near_workspaces::{Account, AccountId, Contract, Worker};
use serde_json::json;

const ACCOUNT_WASM: &[u8] = include_bytes!("../../../res/defuse-account-contract.wasm");
const INTENT_WASM: &[u8] = include_bytes!("../../../res/defuse-intent-contract.wasm");
const TOKEN_WASM: &[u8] = include_bytes!("../../../res/fungible-token.wasm");

const TOTAL_SUPPLY: u128 = 1_000_000_000;

pub struct Sandbox {
    worker: Worker<near_workspaces::network::Sandbox>,
    root_account: Account,
}

impl Sandbox {
    pub async fn new() -> anyhow::Result<Self> {
        let worker = near_workspaces::sandbox().await?;
        let root_account = worker.root_account()?;

        Ok(Self {
            worker,
            root_account,
        })
    }

    pub async fn create_subaccount(
        &self,
        name: &str,
        balance: NearToken,
    ) -> anyhow::Result<Account> {
        self.root_account
            .create_subaccount(name)
            .initial_balance(balance)
            .transact()
            .await
            .map(|result| result.result)
            .map_err(Into::into)
    }

    pub async fn balance(&self, account_id: &AccountId) -> u128 {
        self.worker
            .view_account(account_id)
            .await
            .unwrap()
            .balance
            .as_yoctonear()
    }

    pub async fn deploy_account_contract(&self) -> Contract {
        self.deploy_contract("account", ACCOUNT_WASM).await
    }

    pub async fn deploy_intent_contract(&self) -> Contract {
        let contract = self.deploy_contract("intent", INTENT_WASM).await;
        let result = contract
            .call("new")
            .args_json(json!({
                "owner_id": contract.id()
            }))
            .max_gas()
            .transact()
            .await
            .unwrap();
        assert!(result.is_success(), "{result:?}");

        contract
    }

    pub async fn deploy_token(&self, token: &str) -> Contract {
        let contract = self.deploy_contract(token, TOKEN_WASM).await;
        let result = contract
            .call("new")
            .args_json(json!({
                "owner_id": contract.id(),
                "total_supply": TOTAL_SUPPLY.to_string(),
                "metadata": {
                    "spec": "ft-1.0.0",
                    "name": format!("Token {}", &token),
                    "symbol": "TKN",
                    "decimals": 18
                }
            }))
            .max_gas()
            .transact()
            .await
            .unwrap();
        assert!(result.is_success(), "{result:?}");

        contract
    }

    async fn deploy_contract(&self, account_id: &str, wasm: &[u8]) -> Contract {
        let contract_id = self
            .create_subaccount(account_id, NearToken::from_near(10))
            .await
            .unwrap();
        let result = contract_id.deploy(wasm).await.unwrap();
        assert!(result.is_success());

        result.result
    }
}
