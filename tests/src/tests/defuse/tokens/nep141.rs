use anyhow::anyhow;
use defuse_contracts::defuse::tokens::{DepositMessage, TokenId};
use near_sdk::{json_types::U128, AccountId, NearToken};
use serde_json::json;

use crate::{
    tests::defuse::{env::Env, DefuseExt},
    utils::ft::FtExt,
};

#[tokio::test]
async fn test_deposit_withdraw() {
    let env = Env::new().await.unwrap();

    env.defuse_ft_mint(env.ft1.id(), 1000, env.user1.id().clone(), [])
        .await
        .unwrap();

    let ft1 = TokenId::Nep141(env.ft1.id().clone());

    assert_eq!(
        env.defuse
            .mt_balance_of(env.user1.id(), &ft1)
            .await
            .unwrap(),
        1000
    );

    assert!(env
        .user1
        .defuse_ft_withdraw(env.defuse.id(), env.ft1.id(), None, 1000)
        .await
        .unwrap());

    assert_eq!(
        env.defuse
            .mt_balance_of(env.user1.id(), &ft1)
            .await
            .unwrap(),
        0
    );

    assert_eq!(env.ft1.ft_balance_of(env.user1.id()).await.unwrap(), 1000);
}

pub trait DefuseFtReceiver {
    async fn defuse_ft_deposit(
        &self,
        defuse_id: &AccountId,
        token_id: &AccountId,
        amount: u128,
        msg: &DepositMessage,
    ) -> anyhow::Result<()>;
}

impl DefuseFtReceiver for near_workspaces::Account {
    async fn defuse_ft_deposit(
        &self,
        defuse_id: &AccountId,
        token_id: &AccountId,
        amount: u128,
        msg: &DepositMessage,
    ) -> anyhow::Result<()> {
        if self
            .ft_transfer_call(
                token_id,
                defuse_id,
                amount,
                None,
                &serde_json::to_string(msg)?,
            )
            .await?
            != amount
        {
            return Err(anyhow!("refunded"));
        }
        Ok(())
    }
}

impl DefuseFtReceiver for near_workspaces::Contract {
    async fn defuse_ft_deposit(
        &self,
        defuse_id: &AccountId,
        token_id: &AccountId,
        amount: u128,
        msg: &DepositMessage,
    ) -> anyhow::Result<()> {
        self.as_account()
            .defuse_ft_deposit(defuse_id, token_id, amount, msg)
            .await
    }
}

pub trait DefuseFtWithdrawer {
    async fn defuse_ft_withdraw(
        &self,
        defuse_id: &AccountId,
        token_id: &AccountId,
        to: impl Into<Option<&AccountId>>,
        amount: u128,
    ) -> anyhow::Result<bool>;

    async fn defuse_ft_withdraw_call(
        &self,
        defuse_id: &AccountId,
        token_id: &AccountId,
        to: impl Into<Option<&AccountId>>,
        amount: u128,
        msg: String,
    ) -> anyhow::Result<u128>;
}

impl DefuseFtWithdrawer for near_workspaces::Account {
    async fn defuse_ft_withdraw(
        &self,
        defuse_id: &AccountId,
        token_id: &AccountId,
        to: impl Into<Option<&AccountId>>,
        amount: u128,
    ) -> anyhow::Result<bool> {
        self.call(defuse_id, "nep141_withdraw")
            .deposit(NearToken::from_yoctonear(1))
            .args_json(json!({
                "token_id": token_id,
                "to": to.into(),
                "amount": U128(amount),
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?
            .json()
            .map_err(Into::into)
    }

    async fn defuse_ft_withdraw_call(
        &self,
        defuse_id: &AccountId,
        token_id: &AccountId,
        to: impl Into<Option<&AccountId>>,
        amount: u128,
        msg: String,
    ) -> anyhow::Result<u128> {
        Ok(self
            .call(defuse_id, "nep141_withdraw_call")
            .deposit(NearToken::from_yoctonear(1))
            .args_json(json!({
                "token_id": token_id,
                "to": to.into(),
                "amount": U128(amount),
                "msg": msg,
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?
            .json::<U128>()?
            .0)
    }
}

impl DefuseFtWithdrawer for near_workspaces::Contract {
    async fn defuse_ft_withdraw(
        &self,
        defuse_id: &AccountId,
        token_id: &AccountId,
        to: impl Into<Option<&AccountId>>,
        amount: u128,
    ) -> anyhow::Result<bool> {
        self.as_account()
            .defuse_ft_withdraw(defuse_id, token_id, to, amount)
            .await
    }

    async fn defuse_ft_withdraw_call(
        &self,
        defuse_id: &AccountId,
        token_id: &AccountId,
        to: impl Into<Option<&AccountId>>,
        amount: u128,
        msg: String,
    ) -> anyhow::Result<u128> {
        self.as_account()
            .defuse_ft_withdraw_call(defuse_id, token_id, to, amount, msg)
            .await
    }
}
