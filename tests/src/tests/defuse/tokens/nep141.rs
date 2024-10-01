use anyhow::anyhow;
use defuse_contracts::defuse::tokens::TokenId;
use near_sdk::{json_types::U128, AccountId, NearToken};
use serde_json::json;

use crate::{
    tests::defuse::{env::Env, DefuseExt},
    utils::ft::FtExt,
};

#[tokio::test]
async fn test_deposit_withdraw() {
    let env = Env::new().await.unwrap();

    env.defuse_ft_mint(env.ft1.id(), 1000, env.user1.id())
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

    assert_eq!(
        env.user1
            .defuse_ft_withdraw(env.defuse.id(), env.ft1.id(), env.user1.id(), 1000, None)
            .await
            .unwrap(),
        1000,
    );

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
        to: impl Into<Option<&AccountId>>,
    ) -> anyhow::Result<()>;
}

impl DefuseFtReceiver for near_workspaces::Account {
    async fn defuse_ft_deposit(
        &self,
        defuse_id: &AccountId,
        token_id: &AccountId,
        amount: u128,
        to: impl Into<Option<&AccountId>>,
    ) -> anyhow::Result<()> {
        if self
            .ft_transfer_call(
                token_id,
                defuse_id,
                amount,
                None,
                to.into().map(AsRef::<str>::as_ref).unwrap_or_default(),
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
        to: impl Into<Option<&AccountId>>,
    ) -> anyhow::Result<()> {
        self.as_account()
            .defuse_ft_deposit(defuse_id, token_id, amount, to)
            .await
    }
}

pub trait DefuseFtWithdrawer {
    async fn defuse_ft_withdraw(
        &self,
        defuse_id: &AccountId,
        token_id: &AccountId,
        receiver_id: &AccountId,
        amount: u128,
        msg: Option<String>,
    ) -> anyhow::Result<u128>;
}

impl DefuseFtWithdrawer for near_workspaces::Account {
    async fn defuse_ft_withdraw(
        &self,
        defuse_id: &AccountId,
        token: &AccountId,
        receiver_id: &AccountId,
        amount: u128,
        msg: Option<String>,
    ) -> anyhow::Result<u128> {
        self.call(defuse_id, "ft_withdraw")
            .deposit(NearToken::from_yoctonear(1))
            .args_json(json!({
                "token": token,
                "receiver_id": receiver_id,
                "amount": U128(amount),
                "msg": msg,
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?
            .json::<U128>()
            .map(|v| v.0)
            .map_err(Into::into)
    }
}

impl DefuseFtWithdrawer for near_workspaces::Contract {
    async fn defuse_ft_withdraw(
        &self,
        defuse_id: &AccountId,
        token_id: &AccountId,
        receiver_id: &AccountId,
        amount: u128,
        msg: Option<String>,
    ) -> anyhow::Result<u128> {
        self.as_account()
            .defuse_ft_withdraw(defuse_id, token_id, receiver_id, amount, msg)
            .await
    }
}
