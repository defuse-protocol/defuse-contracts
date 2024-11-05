use core::iter;

use defuse_contracts::{
    defuse::{
        intents::tokens::MtWithdraw,
        tokens::{
            nep245::{MultiTokenForceWithdrawer, MultiTokenWithdrawResolver, MultiTokenWithdrawer},
            TokenId,
        },
        DefuseError, Result,
    },
    nep245,
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

#[near]
impl MultiTokenWithdrawer for DefuseImpl {
    #[pause]
    #[payable]
    fn mt_withdraw(
        &mut self,
        token: AccountId,
        receiver_id: AccountId,
        token_ids: Vec<nep245::TokenId>,
        amounts: Vec<U128>,
        memo: Option<String>,
    ) -> PromiseOrValue<bool> {
        self.internal_mt_withdraw(
            PREDECESSOR_ACCOUNT_ID.clone(),
            token,
            receiver_id,
            token_ids,
            amounts,
            memo,
        )
    }
}

impl DefuseImpl {
    fn internal_mt_withdraw(
        &mut self,
        owner_id: AccountId,
        token: AccountId,
        receiver_id: AccountId,
        token_ids: Vec<nep245::TokenId>,
        amounts: Vec<U128>,
        memo: Option<String>,
    ) -> PromiseOrValue<bool> {
        assert_one_yocto();
        let owner = self
            .accounts
            .get_mut(&owner_id)
            .ok_or(DefuseError::AccountNotFound)
            .unwrap_or_panic();
        self.state
            .mt_withdraw(
                owner_id,
                owner,
                MtWithdraw {
                    token,
                    receiver_id,
                    token_ids,
                    amounts,
                    memo,
                    storage_deposit: None,
                },
            )
            .unwrap_or_panic()
    }
}

impl State {
    pub fn mt_withdraw(
        &mut self,
        sender_id: AccountId,
        sender: &mut Account,
        withdraw: MtWithdraw,
    ) -> Result<PromiseOrValue<bool>> {
        require!(
            withdraw.token_ids.len() == withdraw.amounts.len() && !withdraw.amounts.is_empty(),
            "invalid args"
        );

        self.internal_withdraw(
            &sender_id,
            sender,
            iter::repeat(withdraw.token.clone())
                .zip(withdraw.token_ids.iter().cloned())
                .map(|(token, token_id)| TokenId::Nep245(token, token_id))
                .zip(withdraw.amounts.iter().map(|a| a.0))
                .chain(withdraw.storage_deposit.map(|amount| {
                    (
                        TokenId::Nep141(self.wnear_id.clone()),
                        amount.as_yoctonear(),
                    )
                })),
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
                        .with_static_gas(DefuseImpl::do_mt_withdraw_gas(
                            withdraw
                                .token_ids
                                .len()
                                .try_into()
                                .unwrap_or_else(|_| unreachable!()),
                        ))
                        .do_mt_withdraw(withdraw.clone()),
                )
        } else {
            DefuseImpl::do_mt_withdraw(withdraw.clone())
        }
        .then(
            DefuseImpl::ext(CURRENT_ACCOUNT_ID.clone())
                .with_static_gas(DefuseImpl::mt_resolve_withdraw_gas(
                    withdraw
                        .token_ids
                        .len()
                        .try_into()
                        .unwrap_or_else(|_| unreachable!()),
                ))
                .mt_resolve_withdraw(
                    withdraw.token,
                    sender_id,
                    withdraw.token_ids,
                    withdraw.amounts,
                ),
        )
        .into())
    }
}

#[near]
impl DefuseImpl {
    #[private]
    pub fn do_mt_withdraw(withdraw: MtWithdraw) -> Promise {
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
        .mt_batch_transfer(
            &withdraw.receiver_id,
            &withdraw.token_ids,
            &withdraw.amounts,
            withdraw.memo.as_deref(),
        )
    }

    #[inline]
    const fn do_mt_withdraw_gas(token_count: u64) -> Gas {
        // TODO: more accurate numbers
        const DO_MT_WITHDRAW_GAS_BASE: Gas = Gas::from_tgas(1);
        const DO_MT_WITHDRAW_GAS_PER_TOKEN_ID: Gas = Gas::from_ggas(500);

        DO_MT_WITHDRAW_GAS_BASE
            .saturating_add(DO_MT_WITHDRAW_GAS_PER_TOKEN_ID.saturating_mul(token_count as u64))
            // do_mt_withdraw() method is called externally
            // only with storage_deposit
            .saturating_add(STORAGE_DEPOSIT_GAS)
            .saturating_add(Self::mt_batch_transfer_gas(token_count))
    }

    #[inline]
    const fn mt_batch_transfer_gas(token_count: u64) -> Gas {
        // TODO: more accurate numbers
        const MT_TRANSFER_GAS_BASE: Gas = Gas::from_tgas(15);
        const MT_TRANSFER_GAS_PER_TOKEN_ID: Gas = Gas::from_ggas(500);

        MT_TRANSFER_GAS_BASE
            .saturating_add(MT_TRANSFER_GAS_PER_TOKEN_ID.saturating_mul(token_count))
    }

    #[inline]
    const fn mt_resolve_withdraw_gas(token_count: u64) -> Gas {
        // TODO: more accurate numbers
        const MT_RESOLVE_WITHDRAW_GAS_BASE: Gas = Gas::from_tgas(5);
        const MT_RESOLVE_WITHDRAW_GAS_PER_TOKEN_ID: Gas = Gas::from_ggas(500);

        MT_RESOLVE_WITHDRAW_GAS_BASE
            .saturating_add(MT_RESOLVE_WITHDRAW_GAS_PER_TOKEN_ID.saturating_mul(token_count))
    }
}

#[near]
impl MultiTokenWithdrawResolver for DefuseImpl {
    #[private]
    fn mt_resolve_withdraw(
        &mut self,
        token: AccountId,
        sender_id: AccountId,
        token_ids: Vec<nep245::TokenId>,
        amounts: Vec<U128>,
    ) -> bool {
        require!(
            token_ids.len() == amounts.len() && !amounts.is_empty(),
            "invalid args"
        );

        let ok =
            matches!(env::promise_result(0), PromiseResult::Successful(data) if data.is_empty());

        if !ok {
            self.internal_deposit(
                sender_id,
                iter::repeat(token)
                    .zip(token_ids)
                    .map(|(token, token_id)| TokenId::Nep245(token, token_id))
                    .zip(amounts.into_iter().map(|a| a.0)),
                Some("refund"),
            )
            .unwrap_or_panic();
        }
        ok
    }
}

#[near]
impl MultiTokenForceWithdrawer for DefuseImpl {
    #[access_control_any(roles(Role::DAO, Role::UnrestrictedWithdrawer))]
    #[payable]
    fn mt_force_withdraw(
        &mut self,
        owner_id: AccountId,
        token: AccountId,
        receiver_id: AccountId,
        token_ids: Vec<nep245::TokenId>,
        amounts: Vec<U128>,
        memo: Option<String>,
    ) -> PromiseOrValue<bool> {
        self.internal_mt_withdraw(owner_id, token, receiver_id, token_ids, amounts, memo)
    }
}

pub trait MtExt {
    fn mt_batch_transfer(
        self,
        receiver_id: &AccountId,
        token_ids: &[nep245::TokenId],
        amounts: &[U128],
        memo: Option<&str>,
    ) -> Self;
}

impl MtExt for Promise {
    fn mt_batch_transfer(
        self,
        receiver_id: &AccountId,
        token_ids: &[nep245::TokenId],
        amounts: &[U128],
        memo: Option<&str>,
    ) -> Self {
        self.function_call(
            "mt_batch_transfer".to_string(),
            serde_json::to_vec(&json!({
                "receiver_id": receiver_id,
                "token_ids": token_ids,
                "amounts": amounts,
                "memo": memo,
            }))
            .unwrap_or_panic_display(),
            NearToken::from_yoctonear(1),
            DefuseImpl::mt_batch_transfer_gas(
                token_ids
                    .len()
                    .try_into()
                    .unwrap_or_else(|_| unreachable!()),
            ),
        )
    }
}
