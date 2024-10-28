use std::sync::LazyLock;

use near_sdk::{json_types::U128, AccountId, NearToken};
use near_workspaces::Contract;
use serde_json::json;

use crate::utils::{account::AccountExt, read_wasm};

static POA_FACTORY_WASM: LazyLock<Vec<u8>> =
    LazyLock::new(|| read_wasm("defuse_poa_factory_contract"));

pub trait PoAFactoryExt {
    async fn deploy_poa_factory(&self, name: &str) -> anyhow::Result<Contract>;

    #[track_caller]
    fn token_id(token: &str, factory: &AccountId) -> AccountId {
        format!("{token}.{factory}").parse().unwrap()
    }
    async fn poa_factory_deploy_token(
        &self,
        factory: &AccountId,
        token: &str,
    ) -> anyhow::Result<AccountId>;
    async fn poa_deploy_token(&self, token: &str) -> anyhow::Result<AccountId>;

    async fn poa_factory_ft_mint(
        &self,
        factory: &AccountId,
        token: &str,
        owner_id: &AccountId,
        amount: u128,
        msg: Option<String>,
        memo: Option<String>,
    ) -> anyhow::Result<()>;
    async fn poa_ft_mint(
        &self,
        token: &str,
        owner_id: &AccountId,
        amount: u128,
        msg: Option<String>,
        memo: Option<String>,
    ) -> anyhow::Result<()>;
}

impl PoAFactoryExt for near_workspaces::Account {
    async fn deploy_poa_factory(&self, name: &str) -> anyhow::Result<Contract> {
        let contract = self.deploy_contract(name, &POA_FACTORY_WASM).await?;
        self.transfer_near(contract.id(), NearToken::from_near(100))
            .await?
            .into_result()?;
        contract
            .call("new")
            .args_json(json!({}))
            .max_gas()
            .transact()
            .await?
            .into_result()?;
        Ok(contract)
    }

    async fn poa_factory_deploy_token(
        &self,
        factory: &AccountId,
        token: &str,
    ) -> anyhow::Result<AccountId> {
        self.call(factory, "deploy_token")
            .args_json(json!({
                "token": token,
            }))
            .deposit(NearToken::from_near(4))
            .max_gas()
            .transact()
            .await?
            .into_result()?;
        Ok(Self::token_id(token, factory))
    }

    async fn poa_deploy_token(&self, token: &str) -> anyhow::Result<AccountId> {
        self.poa_factory_deploy_token(self.id(), token).await
    }

    async fn poa_factory_ft_mint(
        &self,
        factory: &AccountId,
        token: &str,
        owner_id: &AccountId,
        amount: u128,
        msg: Option<String>,
        memo: Option<String>,
    ) -> anyhow::Result<()> {
        self.call(factory, "ft_mint")
            .args_json(json!({
                "token": token,
                "owner_id": owner_id,
                "amount": U128(amount),
                "msg": msg,
                "memo": memo,
            }))
            .deposit(NearToken::from_millinear(4))
            .max_gas()
            .transact()
            .await?
            .into_result()?;
        Ok(())
    }

    async fn poa_ft_mint(
        &self,
        token: &str,
        owner_id: &AccountId,
        amount: u128,
        msg: Option<String>,
        memo: Option<String>,
    ) -> anyhow::Result<()> {
        self.poa_factory_ft_mint(self.id(), token, owner_id, amount, msg, memo)
            .await
    }
}

impl PoAFactoryExt for near_workspaces::Contract {
    async fn deploy_poa_factory(&self, name: &str) -> anyhow::Result<Contract> {
        self.as_account().deploy_poa_factory(name).await
    }

    async fn poa_factory_deploy_token(
        &self,
        factory: &AccountId,
        token: &str,
    ) -> anyhow::Result<AccountId> {
        self.as_account()
            .poa_factory_deploy_token(factory, token)
            .await
    }

    async fn poa_deploy_token(&self, token: &str) -> anyhow::Result<AccountId> {
        self.as_account().poa_deploy_token(token).await
    }

    async fn poa_factory_ft_mint(
        &self,
        factory: &AccountId,
        token: &str,
        owner_id: &AccountId,
        amount: u128,
        msg: Option<String>,
        memo: Option<String>,
    ) -> anyhow::Result<()> {
        self.as_account()
            .poa_factory_ft_mint(factory, token, owner_id, amount, msg, memo)
            .await
    }

    async fn poa_ft_mint(
        &self,
        token: &str,
        owner_id: &AccountId,
        amount: u128,
        msg: Option<String>,
        memo: Option<String>,
    ) -> anyhow::Result<()> {
        self.as_account()
            .poa_ft_mint(token, owner_id, amount, msg, memo)
            .await
    }
}
