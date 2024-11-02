use defuse_contracts::{
    defuse::{
        intents::tokens::MtWithdraw,
        tokens::{
            nep245::{MultiTokenWithdrawResolver, MultiTokenWithdrawer},
            TokenId,
        },
        DefuseError, Result,
    },
    nep245,
    utils::{
        cache::{CURRENT_ACCOUNT_ID, PREDECESSOR_ACCOUNT_ID},
        UnwrapOrPanic, UnwrapOrPanicError,
    },
};
use near_contract_standards::storage_management::ext_storage_management;
use near_plugins::{pause, Pausable};
use near_sdk::{
    assert_one_yocto, env,
    json_types::U128,
    near, require,
    serde_json::{self, json},
    AccountId, Gas, NearToken, Promise, PromiseOrValue, PromiseResult,
};

use crate::{
    accounts::Account, state::State, tokens::storage_management::STORAGE_DEPOSIT_GAS, DefuseImpl,
    DefuseImplExt,
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
        storage_deposit: Option<NearToken>,
    ) -> PromiseOrValue<bool> {
        assert_one_yocto();
        let sender_id = PREDECESSOR_ACCOUNT_ID.clone();
        let sender = self
            .accounts
            .get_mut(&sender_id)
            .ok_or(DefuseError::AccountNotFound)
            .unwrap_or_panic();
        self.state
            .mt_withdraw(
                sender_id,
                sender,
                MtWithdraw {
                    token,
                    receiver_id,
                    token_ids,
                    amounts,
                    memo,
                    storage_deposit,
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
        withdraw @ MtWithdraw {
            storage_deposit, ..
        }: MtWithdraw,
    ) -> Result<PromiseOrValue<bool>> {
        if let Some(storage_deposit) = storage_deposit {
            require!(
                withdraw.token_ids.len() == withdraw.amounts.len()
                    && !withdraw.amounts.is_empty()
                    // check all amounts before unwrapping wNEAR
                    && withdraw.amounts.iter().all(|a| a.0 > 0),
                "invalid args"
            );
            Ok(self
                .unwrap_wnear(
                    sender_id.clone(),
                    sender,
                    storage_deposit,
                    Some("storage_deposit"),
                )?
                .then(
                    DefuseImpl::ext(CURRENT_ACCOUNT_ID.clone())
                        .with_static_gas(DefuseImpl::do_mt_withdraw_gas(withdraw.token_ids.len()))
                        .do_mt_withdraw(sender_id, withdraw),
                )
                .into())
        } else {
            self.do_mt_withdraw(sender_id, sender, withdraw)
        }
    }
}

#[near]
impl DefuseImpl {
    #[private]
    pub fn do_mt_withdraw(
        &mut self,
        sender_id: AccountId,
        withdraw: MtWithdraw,
    ) -> PromiseOrValue<bool> {
        if withdraw.storage_deposit.is_some() {
            require!(
                matches!(env::promise_result(0), PromiseResult::Successful(data) if data == b"true"),
                "failed to unwrap wNEAR",
            );
        }

        let sender = self
            .accounts
            .get_mut(&sender_id)
            .ok_or(DefuseError::AccountNotFound)
            .unwrap_or_panic();
        self.state
            .do_mt_withdraw(sender_id, sender, withdraw)
            .unwrap_or_panic()
    }

    #[inline]
    const fn do_mt_withdraw_gas(token_count: usize) -> Gas {
        // TODO: more accurate numbers
        const DO_MT_WITHDRAW_GAS_BASE: Gas = Gas::from_tgas(10);
        const DO_MT_WITHDRAW_GAS_PER_TOKEN_ID: Gas = Gas::from_ggas(500);

        DO_MT_WITHDRAW_GAS_BASE
            .saturating_add(DO_MT_WITHDRAW_GAS_PER_TOKEN_ID.saturating_mul(token_count as u64))
            .saturating_add(STORAGE_DEPOSIT_GAS)
            .saturating_add(Self::mt_batch_transfer_gas(token_count))
            .saturating_add(Self::mt_resolve_withdraw_gas(token_count))
    }

    #[inline]
    const fn mt_batch_transfer_gas(token_count: usize) -> Gas {
        // TODO: more accurate numbers
        const MT_TRANSFER_GAS_BASE: Gas = Gas::from_tgas(15);
        const MT_TRANSFER_GAS_PER_TOKEN_ID: Gas = Gas::from_ggas(500);

        MT_TRANSFER_GAS_BASE
            .saturating_add(MT_TRANSFER_GAS_PER_TOKEN_ID.saturating_mul(token_count as u64))
    }

    #[inline]
    const fn mt_resolve_withdraw_gas(token_count: usize) -> Gas {
        // TODO: more accurate numbers
        const MT_RESOLVE_WITHDRAW_GAS_BASE: Gas = Gas::from_tgas(5);
        const MT_RESOLVE_WITHDRAW_GAS_PER_TOKEN_ID: Gas = Gas::from_ggas(500);

        MT_RESOLVE_WITHDRAW_GAS_BASE
            .saturating_add(MT_RESOLVE_WITHDRAW_GAS_PER_TOKEN_ID.saturating_mul(token_count as u64))
    }
}

impl State {
    fn do_mt_withdraw(
        &mut self,
        sender_id: AccountId,
        sender: &mut Account,
        MtWithdraw {
            token,
            receiver_id,
            token_ids,
            amounts,
            memo,
            storage_deposit,
        }: MtWithdraw,
    ) -> Result<PromiseOrValue<bool>> {
        require!(
            token_ids.len() == amounts.len() && !amounts.is_empty(),
            "invalid args"
        );

        self.internal_withdraw(
            &sender_id,
            sender,
            token_ids
                .iter()
                .cloned()
                .map(|token_id| TokenId::Nep245(token.clone(), token_id))
                .zip(amounts.iter().map(|a| a.0)),
            Some("withdraw"),
        )?;

        Ok(if let Some(storage_deposit) = storage_deposit {
            ext_storage_management::ext(token.clone())
                .with_attached_deposit(storage_deposit)
                .with_static_gas(STORAGE_DEPOSIT_GAS)
                .storage_deposit(Some(receiver_id.clone()), None)
        } else {
            Promise::new(token.clone())
        }
        .mt_batch_transfer(&receiver_id, &token_ids, &amounts, memo.as_deref())
        .then(
            DefuseImpl::ext(CURRENT_ACCOUNT_ID.clone())
                .with_static_gas(DefuseImpl::mt_resolve_withdraw_gas(token_ids.len()))
                .mt_resolve_withdraw(token, sender_id, token_ids, amounts),
        )
        .into())
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
                token_ids
                    .iter()
                    .cloned()
                    .map(|token_id| TokenId::Nep245(token.clone(), token_id))
                    .zip(amounts.iter().map(|a| a.0)),
                Some("refund"),
            )
            .unwrap_or_panic();
        }
        ok
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
            DefuseImpl::mt_batch_transfer_gas(token_ids.len()),
        )
    }
}
