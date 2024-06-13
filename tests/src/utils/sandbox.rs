use std::fs;
use std::path::Path;

use lazy_static::lazy_static;
use near_workspaces::{types::NearToken, Account, AccountId, Contract, Worker};
use serde_json::json;

fn read_wasm(name: impl AsRef<Path>) -> Vec<u8> {
    let filename = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../res/")
        .join(name)
        .with_extension("wasm");
    fs::read(filename).unwrap()
}
lazy_static! {
    static ref ACCOUNT_WASM: Vec<u8> = read_wasm("defuse-account-contract");
    static ref CONTROLLER_WASM: Vec<u8> = read_wasm("defuse-controller-contract");
    static ref INTENT_WASM: Vec<u8> = read_wasm("defuse-intent-contract");
    static ref FUNGIBLE_TOKEN_WASM: Vec<u8> = read_wasm("fungible-token");
}

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

    pub async fn create_account(&self, name: &str) -> Account {
        self.create_subaccount(name, NearToken::from_near(10))
            .await
            .unwrap()
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
        let contract = self.deploy_contract("account", &ACCOUNT_WASM).await;
        let result = contract
            .call("new")
            .args_json(json!({
                "owner_id": "controller.test.near",
                "mpc_contract_id": "mpc.test.net"
            }))
            .max_gas()
            .transact()
            .await
            .unwrap();
        assert!(result.is_success(), "{result:#?}");

        contract
    }

    pub async fn deploy_controller_contract(&self) -> Contract {
        let contract = self.deploy_contract("controller", &CONTROLLER_WASM).await;
        let result = contract
            .call("new")
            .args_json(json!({
                "owner_id": "dao.test.near"
            }))
            .max_gas()
            .transact()
            .await
            .unwrap();
        assert!(result.is_success(), "{result:#?}");

        contract
    }

    pub async fn deploy_intent_contract(&self) -> Contract {
        let contract = self.deploy_contract("intent", &INTENT_WASM).await;
        let result = contract
            .call("new")
            .args_json(json!({
                "owner_id": contract.id()
            }))
            .max_gas()
            .transact()
            .await
            .unwrap();
        assert!(result.is_success(), "{result:#?}");

        contract
    }

    pub async fn deploy_token(&self, token: &str) -> Contract {
        let contract = self.deploy_contract(token, &FUNGIBLE_TOKEN_WASM).await;
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
        assert!(result.is_success(), "{result:#?}");

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
