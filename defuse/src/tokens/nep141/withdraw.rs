use core::iter;

use defuse_contracts::{
    defuse::{
        intents::tokens::FtWithdraw,
        tokens::{
            nep141::{
                FungibleTokenForceWithdrawer, FungibleTokenWithdrawResolver,
                FungibleTokenWithdrawer,
            },
            TokenId,
        },
        DefuseError, Result,
    },
    utils::{
        cache::{CURRENT_ACCOUNT_ID, PREDECESSOR_ACCOUNT_ID},
        UnwrapOrPanic, UnwrapOrPanicError,
    },
    wnear::{ext_wnear, NEAR_WITHDRAW_GAS},
};
use near_contract_standards::storage_management::ext_storage_management;
use near_plugins::{access_control_any, pause, AccessControllable, Pausable};
use near_sdk::{
    assert_one_yocto, env,
    json_types::U128,
    near, require,
    serde_json::{self, json},
    AccountId, Gas, NearToken, Promise, PromiseOrValue, PromiseResult,
};

use crate::{
    accounts::Account, state::State, tokens::STORAGE_DEPOSIT_GAS, DefuseImpl, DefuseImplExt, Role,
};

const FT_TRANSFER_GAS: Gas = Gas::from_tgas(15);

#[near]
impl FungibleTokenWithdrawer for DefuseImpl {
    #[pause]
    #[payable]
    fn ft_withdraw(
        &mut self,
        token: AccountId,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
    ) -> PromiseOrValue<bool> {
        self.internal_ft_withdraw(
            PREDECESSOR_ACCOUNT_ID.clone(),
            token,
            receiver_id,
            amount,
            memo,
        )
    }
}

impl DefuseImpl {
    fn internal_ft_withdraw(
        &mut self,
        owner_id: AccountId,
        token: AccountId,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
    ) -> PromiseOrValue<bool> {
        assert_one_yocto();
        let owner = self
            .accounts
            .get_mut(&owner_id)
            .ok_or(DefuseError::AccountNotFound)
            .unwrap_or_panic();
        self.state
            .ft_withdraw(
                owner_id,
                owner,
                FtWithdraw {
                    token,
                    receiver_id,
                    amount,
                    memo,
                    storage_deposit: None,
                },
            )
            .unwrap_or_panic()
    }
}

impl State {
    pub fn ft_withdraw(
        &mut self,
        sender_id: AccountId,
        sender: &mut Account,
        withdraw: FtWithdraw,
    ) -> Result<PromiseOrValue<bool>> {
        self.internal_withdraw(
            &sender_id,
            sender,
            iter::once((TokenId::Nep141(withdraw.token.clone()), withdraw.amount.0)).chain(
                withdraw.storage_deposit.map(|amount| {
                    (
                        TokenId::Nep141(self.wnear_id.clone()),
                        amount.as_yoctonear(),
                    )
                }),
            ),
            Some("withdraw"),
        )?;

        Ok(if let Some(storage_deposit) = withdraw.storage_deposit {
            ext_wnear::ext(self.wnear_id.clone())
                .with_attached_deposit(NearToken::from_yoctonear(1))
                .with_static_gas(NEAR_WITHDRAW_GAS)
                .near_withdraw(U128(storage_deposit.as_yoctonear()))
                .then(
                    // schedule storage_deposit() only after near_withdraw() returns
                    DefuseImpl::ext(CURRENT_ACCOUNT_ID.clone())
                        .with_static_gas(DefuseImpl::DO_FT_WITHDRAW_GAS)
                        .do_ft_withdraw(withdraw.clone()),
                )
        } else {
            DefuseImpl::do_ft_withdraw(withdraw.clone())
        }
        .then(
            DefuseImpl::ext(CURRENT_ACCOUNT_ID.clone())
                .with_static_gas(DefuseImpl::FT_RESOLVE_WITHDRAW_GAS)
                .ft_resolve_withdraw(withdraw.token, sender_id, withdraw.amount),
        )
        .into())
    }
}

#[near]
impl DefuseImpl {
    const FT_RESOLVE_WITHDRAW_GAS: Gas = Gas::from_tgas(5);
    const DO_FT_WITHDRAW_GAS: Gas = Gas::from_tgas(3)
        // do_ft_withdraw() method is called externally
        // only with storage_deposit
        .saturating_add(STORAGE_DEPOSIT_GAS)
        .saturating_add(FT_TRANSFER_GAS);

    #[private]
    pub fn do_ft_withdraw(withdraw: FtWithdraw) -> Promise {
        if let Some(storage_deposit) = withdraw.storage_deposit {
            require!(
                matches!(env::promise_result(0), PromiseResult::Successful(data) if data.is_empty()),
                "near_withdraw failed",
            );

            ext_storage_management::ext(withdraw.token)
                .with_attached_deposit(storage_deposit)
                .with_static_gas(STORAGE_DEPOSIT_GAS)
                .storage_deposit(Some(withdraw.receiver_id.clone()), None)
        } else {
            Promise::new(withdraw.token)
        }
        .ft_transfer(
            &withdraw.receiver_id,
            withdraw.amount.0,
            withdraw.memo.as_deref(),
        )
    }
}

#[near]
impl FungibleTokenWithdrawResolver for DefuseImpl {
    #[private]
    fn ft_resolve_withdraw(
        &mut self,
        token: AccountId,
        sender_id: AccountId,
        amount: U128,
    ) -> bool {
        let ok =
            matches!(env::promise_result(0), PromiseResult::Successful(data) if data.is_empty());

        if !ok {
            self.internal_deposit(
                sender_id,
                [(TokenId::Nep141(token), amount.0)],
                Some("refund"),
            )
            .unwrap_or_panic();
        }

        ok
    }
}

#[near]
impl FungibleTokenForceWithdrawer for DefuseImpl {
    #[access_control_any(roles(Role::DAO, Role::UnrestrictedWithdrawer))]
    #[payable]
    fn ft_force_withdraw(
        &mut self,
        owner_id: AccountId,
        token: AccountId,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
    ) -> PromiseOrValue<bool> {
        self.internal_ft_withdraw(owner_id, token, receiver_id, amount, memo)
    }
}

pub trait FtExt {
    fn ft_transfer(self, receiver_id: &AccountId, amount: u128, memo: Option<&str>) -> Self;
}

impl FtExt for Promise {
    fn ft_transfer(self, receiver_id: &AccountId, amount: u128, memo: Option<&str>) -> Self {
        self.function_call(
            "ft_transfer".to_string(),
            serde_json::to_vec(&json!({
                "receiver_id": receiver_id,
                "amount": U128(amount),
                "memo": memo,
            }))
            .unwrap_or_panic_display(),
            NearToken::from_yoctonear(1),
            FT_TRANSFER_GAS,
        )
    }
}
