use defuse_contracts::{
    defuse::{
        intents::tokens::{MtWithdraw, StorageDeposit},
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
use near_plugins::{pause, Pausable};
use near_sdk::{
    assert_one_yocto, env,
    json_types::U128,
    near, require,
    serde_json::{self, json},
    AccountId, Gas, NearToken, Promise, PromiseOrValue, PromiseResult,
};

use crate::{accounts::Account, state::State, DefuseImpl, DefuseImplExt};

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
        assert_one_yocto();
        self.internal_mt_withdraw(
            PREDECESSOR_ACCOUNT_ID.clone(),
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

impl DefuseImpl {
    // TODO: export as #[private] for a backup?
    fn internal_mt_withdraw(
        &mut self,
        sender_id: AccountId,
        withdraw: MtWithdraw,
    ) -> Result<PromiseOrValue<bool>> {
        let sender = self
            .accounts
            .get_mut(&sender_id)
            .ok_or(DefuseError::AccountNotFound)?;
        self.state.mt_withdraw(sender_id, sender, withdraw)
    }

    #[inline]
    fn mt_transfer_gas(token_count: usize) -> Gas {
        // TODO: more accurate numbers
        const MT_TRANSFER_GAS_BASE: Gas = Gas::from_tgas(10);
        const MT_TRANSFER_GAS_PER_TOKEN_ID: Gas = Gas::from_tgas(1);

        // if these conversions overflow, then
        // it should have exceeded gas before
        MT_TRANSFER_GAS_BASE
            .checked_add(
                MT_TRANSFER_GAS_PER_TOKEN_ID
                    .checked_mul(token_count as u64)
                    .unwrap_or_else(|| unreachable!()),
            )
            .unwrap_or_else(|| unreachable!())
    }

    #[inline]
    fn mt_resolve_withdraw_gas(token_count: usize) -> Gas {
        // TODO: more accurate numbers
        const MT_RESOLVE_WITHDRAW_GAS_BASE: Gas = Gas::from_tgas(5);
        const MT_RESOLVE_WITHDRAW_GAS_PER_TOKEN_ID: Gas = Gas::from_tgas(1);

        // if these conversions overflow, then
        // it should have exceeded gas before
        MT_RESOLVE_WITHDRAW_GAS_BASE
            .checked_add(
                MT_RESOLVE_WITHDRAW_GAS_PER_TOKEN_ID
                    .checked_mul(token_count as u64)
                    .unwrap_or_else(|| unreachable!()),
            )
            .unwrap_or_else(|| unreachable!())
    }
}

impl State {
    pub fn mt_withdraw(
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
            token_ids.len() == amounts.len(),
            "token_ids.len() != amounts.len()"
        );
        require!(!amounts.is_empty(), "zero amounts");

        self.internal_withdraw(
            &sender_id,
            sender,
            token_ids
                .iter()
                .cloned()
                .map(|token_id| TokenId::Nep245(token.clone(), token_id))
                .zip(amounts.iter().map(|a| a.0))
                .chain(storage_deposit.map(|amount| {
                    (
                        TokenId::Nep141(self.wnear_id.clone()),
                        amount.as_yoctonear(),
                    )
                })),
            Some("withdraw"),
        )?;

        Ok(if let Some(storage_deposit) = storage_deposit {
            self.internal_storage_deposit(StorageDeposit {
                contract_id: token.clone(),
                account_id: receiver_id.clone(),
                amount: storage_deposit,
            })
        } else {
            Promise::new(token.clone())
        }
        .function_call(
            "mt_batch_transfer".to_string(),
            serde_json::to_vec(&json!({
                "receiver_id": &receiver_id,
                "token_ids": &token_ids,
                "amounts": &amounts,
                "memo": &memo,
            }))
            .unwrap_or_panic_display(),
            NearToken::from_yoctonear(1),
            DefuseImpl::mt_transfer_gas(token_ids.len()),
        )
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
