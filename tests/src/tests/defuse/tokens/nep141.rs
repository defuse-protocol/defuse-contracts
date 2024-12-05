use defuse::{
    contract::Role,
    core::{
        intents::{tokens::FtWithdraw, DefuseIntents},
        tokens::TokenId,
        Deadline,
    },
    tokens::DepositMessage,
};
use near_sdk::{json_types::U128, AccountId, NearToken};
use rand::{thread_rng, Rng};
use serde_json::json;

use crate::{
    tests::{
        defuse::{env::Env, DefuseSigner},
        poa::factory::PoAFactoryExt,
    },
    utils::{acl::AclExt, ft::FtExt, mt::MtExt},
};

#[tokio::test]
async fn test_deposit_withdraw() {
    let env = Env::new().await;

    env.defuse_ft_mint(&env.ft1, 1000, env.user1.id())
        .await
        .unwrap();

    let ft1 = TokenId::Nep141(env.ft1.clone());

    assert_eq!(
        env.mt_contract_balance_of(env.defuse.id(), env.user1.id(), &ft1.to_string())
            .await
            .unwrap(),
        1000
    );

    assert!(env
        .user1
        .defuse_ft_withdraw(env.defuse.id(), &env.ft1, env.user1.id(), 1000)
        .await
        .unwrap());

    assert_eq!(
        env.mt_contract_balance_of(env.defuse.id(), env.user1.id(), &ft1.to_string())
            .await
            .unwrap(),
        0
    );

    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.user1.id())
            .await
            .unwrap(),
        1000
    );
}

#[tokio::test]
async fn test_poa_deposit() {
    let env = Env::new().await;
    let ft1 = TokenId::Nep141(env.ft1.clone());

    env.poa_factory_ft_deposit(
        env.poa_factory.id(),
        env.poa_ft1_name(),
        env.user1.id(),
        1000,
        Some(
            DepositMessage {
                receiver_id: env.user1.id().clone(),
                execute_intents: Default::default(),
                refund_if_fails: Default::default(),
            }
            .to_string(),
        ),
        None,
    )
    .await
    .unwrap();

    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.user1.id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.mt_contract_balance_of(env.defuse.id(), env.user1.id(), &ft1.to_string())
            .await
            .unwrap(),
        0
    );
}

#[tokio::test]
async fn test_deposit_withdraw_intent() {
    let env = Env::new().await;

    env.poa_factory_ft_deposit(
        env.poa_factory.id(),
        env.poa_ft1_name(),
        env.user1.id(),
        1000,
        None,
        None,
    )
    .await
    .unwrap();

    assert_eq!(
        env.user1
            .defuse_ft_deposit(
                env.defuse.id(),
                &env.ft1,
                1000,
                DepositMessage {
                    receiver_id: env.user1.id().clone(),
                    execute_intents: [env.user1.sign_defuse_message(
                        env.defuse.id(),
                        thread_rng().gen(),
                        Deadline::MAX,
                        DefuseIntents {
                            intents: [
                                // withdrawal is a detached promise
                                FtWithdraw {
                                    token: env.ft1.clone(),
                                    receiver_id: env.user2.id().clone(),
                                    amount: U128(1000),
                                    memo: None,
                                    storage_deposit: None,
                                }
                                .into(),
                            ]
                            .into(),
                        },
                    )]
                    .into(),
                    // another promise will be created for `execute_intents()`
                    refund_if_fails: false,
                },
            )
            .await
            .unwrap(),
        1000
    );

    let ft1 = TokenId::Nep141(env.ft1.clone());

    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.user1.id())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.mt_contract_balance_of(env.defuse.id(), env.user1.id(), &ft1.to_string())
            .await
            .unwrap(),
        0
    );

    assert_eq!(
        env.mt_contract_balance_of(env.defuse.id(), env.user2.id(), &ft1.to_string())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.user2.id())
            .await
            .unwrap(),
        1000
    );
}

#[tokio::test]
async fn test_deposit_withdraw_intent_refund() {
    let env = Env::new().await;

    env.poa_factory_ft_deposit(
        env.poa_factory.id(),
        env.poa_ft1_name(),
        env.user1.id(),
        1000,
        None,
        None,
    )
    .await
    .unwrap();

    assert_eq!(
        env.user1
            .defuse_ft_deposit(
                env.defuse.id(),
                &env.ft1,
                1000,
                DepositMessage {
                    receiver_id: env.user1.id().clone(),
                    execute_intents: [env.user1.sign_defuse_message(
                        env.defuse.id(),
                        thread_rng().gen(),
                        Deadline::MAX,
                        DefuseIntents {
                            intents: [FtWithdraw {
                                token: env.ft1.clone(),
                                receiver_id: env.user1.id().clone(),
                                amount: U128(1001),
                                memo: None,
                                storage_deposit: None,
                            }
                            .into(),]
                            .into(),
                        },
                    )]
                    .into(),
                    refund_if_fails: true,
                },
            )
            .await
            .unwrap(),
        0
    );

    let ft1 = TokenId::Nep141(env.ft1.clone());
    assert_eq!(
        env.mt_contract_balance_of(env.defuse.id(), env.user1.id(), &ft1.to_string())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.user1.id())
            .await
            .unwrap(),
        1000
    );
}

#[tokio::test]
async fn test_ft_force_withdraw() {
    let env = Env::builder().deployer_as_super_admin().build().await;
    env.defuse_ft_mint(&env.ft1, 1000, env.user1.id())
        .await
        .unwrap();

    let ft1 = TokenId::Nep141(env.ft1.clone());

    env.user2
        .defuse_ft_force_withdraw(
            env.defuse.id(),
            env.user1.id(),
            &env.ft1,
            env.user2.id(),
            1000,
        )
        .await
        .unwrap_err();

    assert_eq!(
        env.mt_contract_balance_of(env.defuse.id(), env.user1.id(), &ft1.to_string())
            .await
            .unwrap(),
        1000
    );
    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.user2.id())
            .await
            .unwrap(),
        0
    );

    env.acl_grant_role(
        env.defuse.id(),
        Role::UnrestrictedWithdrawer,
        env.user2.id(),
    )
    .await
    .unwrap();

    assert!(env
        .user2
        .defuse_ft_force_withdraw(
            env.defuse.id(),
            env.user1.id(),
            &env.ft1,
            env.user2.id(),
            1000
        )
        .await
        .unwrap());

    assert_eq!(
        env.mt_contract_balance_of(env.defuse.id(), env.user1.id(), &ft1.to_string())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        env.ft_token_balance_of(&env.ft1, env.user2.id())
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
        msg: impl Into<Option<DepositMessage>>,
    ) -> anyhow::Result<u128>;
}

impl DefuseFtReceiver for near_workspaces::Account {
    async fn defuse_ft_deposit(
        &self,
        defuse_id: &AccountId,
        token_id: &AccountId,
        amount: u128,
        msg: impl Into<Option<DepositMessage>>,
    ) -> anyhow::Result<u128> {
        self.ft_transfer_call(
            token_id,
            defuse_id,
            amount,
            None,
            &msg.into()
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
        )
        .await
    }
}

impl DefuseFtReceiver for near_workspaces::Contract {
    async fn defuse_ft_deposit(
        &self,
        defuse_id: &AccountId,
        token_id: &AccountId,
        amount: u128,
        msg: impl Into<Option<DepositMessage>>,
    ) -> anyhow::Result<u128> {
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
        receiver_id: &AccountId,
        amount: u128,
    ) -> anyhow::Result<bool>;

    async fn defuse_ft_force_withdraw(
        &self,
        defuse_id: &AccountId,
        owner_id: &AccountId,
        token_id: &AccountId,
        receiver_id: &AccountId,
        amount: u128,
    ) -> anyhow::Result<bool>;
}

impl DefuseFtWithdrawer for near_workspaces::Account {
    async fn defuse_ft_withdraw(
        &self,
        defuse_id: &AccountId,
        token: &AccountId,
        receiver_id: &AccountId,
        amount: u128,
    ) -> anyhow::Result<bool> {
        self.call(defuse_id, "ft_withdraw")
            .deposit(NearToken::from_yoctonear(1))
            .args_json(json!({
                "token": token,
                "receiver_id": receiver_id,
                "amount": U128(amount),
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?
            .json()
            .map_err(Into::into)
    }

    async fn defuse_ft_force_withdraw(
        &self,
        defuse_id: &AccountId,
        owner_id: &AccountId,
        token: &AccountId,
        receiver_id: &AccountId,
        amount: u128,
    ) -> anyhow::Result<bool> {
        self.call(defuse_id, "ft_force_withdraw")
            .deposit(NearToken::from_yoctonear(1))
            .args_json(json!({
                "owner_id": owner_id,
                "token": token,
                "receiver_id": receiver_id,
                "amount": U128(amount),
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?
            .json()
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
    ) -> anyhow::Result<bool> {
        self.as_account()
            .defuse_ft_withdraw(defuse_id, token_id, receiver_id, amount)
            .await
    }

    async fn defuse_ft_force_withdraw(
        &self,
        defuse_id: &AccountId,
        owner_id: &AccountId,
        token_id: &AccountId,
        receiver_id: &AccountId,
        amount: u128,
    ) -> anyhow::Result<bool> {
        self.as_account()
            .defuse_ft_force_withdraw(defuse_id, owner_id, token_id, receiver_id, amount)
            .await
    }
}
