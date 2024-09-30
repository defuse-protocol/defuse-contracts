use defuse_contracts::{
    defuse::{
        tokens::{
            nep245::{MultiTokenWithdrawResolver, MultiTokenWithdrawer},
            TokenId,
        },
        Result,
    },
    nep245::{self, ext_mt_core},
    utils::{
        cache::{CURRENT_ACCOUNT_ID, PREDECESSOR_ACCOUNT_ID},
        UnwrapOrPanic,
    },
};
use near_sdk::{
    assert_one_yocto, env, json_types::U128, near, require, serde_json, AccountId, Gas, NearToken,
    PromiseOrValue, PromiseResult,
};

use crate::{DefuseImpl, DefuseImplExt};

#[near]
impl MultiTokenWithdrawer for DefuseImpl {
    #[payable]
    fn mt_withdraw(
        &mut self,
        token: AccountId,
        receiver_id: AccountId,
        token_ids: Vec<nep245::TokenId>,
        amounts: Vec<U128>,
        memo: Option<String>,
        msg: Option<String>,
    ) -> PromiseOrValue<Vec<U128>> {
        assert_one_yocto();
        self.internal_mt_withdraw(
            PREDECESSOR_ACCOUNT_ID.clone(),
            receiver_id,
            token,
            token_ids,
            amounts,
            memo,
            msg,
        )
        .unwrap_or_panic()
    }
}

impl DefuseImpl {
    // TODO: more accurate numbers
    const MT_RESOLVE_WITHDRAW_GAS_BASE: Gas = Gas::from_tgas(5);
    const MT_RESOLVE_WITHDRAW_GAS_PER_TOKEN_ID: Gas = Gas::from_tgas(1);

    #[allow(clippy::too_many_arguments)]
    fn internal_mt_withdraw(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        token: AccountId,
        token_ids: Vec<nep245::TokenId>,
        amounts: Vec<U128>,
        memo: Option<String>,
        msg: Option<String>,
    ) -> Result<PromiseOrValue<Vec<U128>>> {
        require!(
            token_ids.len() == amounts.len(),
            "token_ids should be the same length as amounts"
        );

        self.internal_withdraw(
            &sender_id,
            token_ids
                .iter()
                .cloned()
                .map(|token_id| TokenId::Nep245(token.clone(), token_id))
                .zip(amounts.iter().map(|a| a.0)),
        )?;

        let ext =
            ext_mt_core::ext(token.clone()).with_attached_deposit(NearToken::from_yoctonear(1));
        let is_call = msg.is_some();
        Ok(if let Some(msg) = msg {
            ext.mt_batch_transfer_call(
                receiver_id,
                token_ids.clone(),
                amounts.clone(),
                None,
                memo,
                msg,
            )
        } else {
            ext.mt_batch_transfer(receiver_id, token_ids.clone(), amounts.clone(), None, memo)
        }
        .then(
            Self::ext(CURRENT_ACCOUNT_ID.clone())
                .with_static_gas(Self::mt_resolve_withdraw_gas(&token_ids))
                .mt_resolve_withdraw(token, sender_id, token_ids, amounts, is_call),
        )
        .into())
    }

    fn mt_resolve_withdraw_gas(#[allow(clippy::ptr_arg)] token_ids: &Vec<String>) -> Gas {
        // if this conversios overflow, then
        // it should have exceeded gas before
        Self::MT_RESOLVE_WITHDRAW_GAS_BASE
            .checked_add(
                Self::MT_RESOLVE_WITHDRAW_GAS_PER_TOKEN_ID
                    .checked_mul(token_ids.len() as u64)
                    .unwrap_or_else(|| unreachable!()),
            )
            .unwrap_or_else(|| unreachable!())
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
        is_call: bool,
    ) -> Vec<U128> {
        require!(
            token_ids.len() == amounts.len(),
            "token_ids should be the same length as amounts"
        );

        let mut used = match env::promise_result(0) {
            PromiseResult::Successful(value) => {
                if is_call {
                    // `mt_batch_transfer_call` returns successfully transferred amounts
                    serde_json::from_slice::<Vec<U128>>(&value)
                        .ok()
                        .filter(|used| used.len() == amounts.len())
                        .unwrap_or_else(|| vec![U128(0); amounts.len()])
                } else {
                    // `mt_batch_transfer` returns empty result on success
                    amounts.clone()
                }
            }
            PromiseResult::Failed => vec![U128(0); amounts.len()],
        };

        let account = self.accounts.get_or_create(sender_id);
        for (token_id, (amount, used)) in token_ids
            .into_iter()
            .zip(amounts.into_iter().zip(&mut used))
        {
            // update min during iteration
            used.0 = used.0.min(amount.0);

            let refund = amount.0 - used.0;
            if refund > 0 {
                let token_id = TokenId::Nep245(token.clone(), token_id);
                self.total_supplies
                    .deposit(token_id.clone(), refund)
                    .unwrap_or_panic();
                account
                    .token_balances
                    .deposit(token_id, refund)
                    .unwrap_or_panic();
            }
        }

        used
    }
}
