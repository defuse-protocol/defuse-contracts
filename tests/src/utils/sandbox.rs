use std::fs;
use std::path::Path;

use lazy_static::lazy_static;
use near_workspaces::{types::NearToken, Account, Network, Worker};

pub fn read_wasm(name: impl AsRef<Path>) -> Vec<u8> {
    let filename = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../res/")
        .join(name)
        .with_extension("wasm");
    fs::read(filename).unwrap()
}
lazy_static! {
    static ref CONTROLLER_WASM: Vec<u8> = read_wasm("defuse-controller-contract");
    static ref MPC_INTENT_WASM: Vec<u8> = read_wasm("defuse-mpc-intent-contract");
}

pub struct Sandbox {
    worker: Worker<near_workspaces::network::Sandbox>,
    root_account: Account,
}

#[allow(dead_code)]
impl Sandbox {
    pub async fn new() -> anyhow::Result<Self> {
        let worker = near_workspaces::sandbox().await?;
        let root_account = worker.root_account()?;

        Ok(Self {
            worker,
            root_account,
        })
    }

    pub const fn worker(&self) -> &Worker<impl Network> {
        &self.worker
    }

    pub const fn root_account(&self) -> &Account {
        &self.root_account
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
}
