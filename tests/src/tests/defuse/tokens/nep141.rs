use anyhow::anyhow;
use defuse_contracts::defuse::token::{DepositMessage, TokenId};
use near_sdk::AccountId;

use crate::{
    tests::defuse::{env::Env, DefuseExt},
    utils::ft::FtExt,
};

#[tokio::test]
async fn test_deposit() {
    let env = Env::new().await.unwrap();

    env.defuse_ft_mint(env.ft1.id(), 1000, env.user1.id().clone(), [].into())
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
