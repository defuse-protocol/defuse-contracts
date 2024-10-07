use defuse_contracts::{
    defuse::{
        intents::tokens::Nep245Withdraw,
        tokens::{
            nep245::{MultiTokenWithdrawResolver, MultiTokenWithdrawer},
            TokenId,
        },
        DefuseError, Result,
    },
    nep245::{self, ext_mt_core},
    utils::{
        cache::{CURRENT_ACCOUNT_ID, PREDECESSOR_ACCOUNT_ID},
        UnwrapOrPanic,
    },
};
use near_sdk::{
    assert_one_yocto, env, json_types::U128, near, serde_json, AccountId, Gas, NearToken, Promise,
    PromiseOrValue, PromiseResult,
};

use crate::{accounts::Account, intents::runtime::Runtime, DefuseImpl, DefuseImplExt};

#[near]
impl MultiTokenWithdrawer for DefuseImpl {
    #[payable]
    fn mt_withdraw(
        &mut self,
        token: AccountId,
        receiver_id: AccountId,
        token_id_amounts: Vec<(nep245::TokenId, U128)>,
        memo: Option<String>,
        msg: Option<String>,
        gas: Option<Gas>,
    ) -> PromiseOrValue<Vec<U128>> {
        assert_one_yocto();
        self.internal_mt_withdraw(
            PREDECESSOR_ACCOUNT_ID.clone(),
            Nep245Withdraw {
                token,
                receiver_id,
                token_id_amounts,
                memo,
                msg,
                gas,
            },
        )
        .unwrap_or_panic()
    }
}

impl DefuseImpl {
    // TODO: more accurate numbers
    const MT_RESOLVE_WITHDRAW_GAS_BASE: Gas = Gas::from_tgas(5);
    const MT_RESOLVE_WITHDRAW_GAS_PER_TOKEN_ID: Gas = Gas::from_tgas(1);

    fn internal_mt_withdraw(
        &mut self,
        sender_id: AccountId,
        withdraw: Nep245Withdraw,
    ) -> Result<PromiseOrValue<Vec<U128>>> {
        let sender = self
            .accounts
            .get_mut(&sender_id)
            .ok_or(DefuseError::AccountNotFound)?;

        Runtime::new(&self.fees, &mut self.total_supplies)
            .mt_withdraw(sender_id, sender, withdraw)
            .map(Into::into)
    }

    #[inline]
    fn mt_resolve_withdraw_gas(
        #[allow(clippy::ptr_arg)] token_id_amounts: &Vec<(nep245::TokenId, U128)>,
    ) -> Gas {
        // if this conversios overflow, then
        // it should have exceeded gas before
        Self::MT_RESOLVE_WITHDRAW_GAS_BASE
            .checked_add(
                Self::MT_RESOLVE_WITHDRAW_GAS_PER_TOKEN_ID
                    .checked_mul(token_id_amounts.len() as u64)
                    .unwrap_or_else(|| unreachable!()),
            )
            .unwrap_or_else(|| unreachable!())
    }
}

impl<'a> Runtime<'a> {
    pub fn mt_withdraw(
        &mut self,
        sender_id: AccountId,
        sender: &mut Account,
        Nep245Withdraw {
            token,
            receiver_id,
            token_id_amounts,
            memo,
            msg,
            gas,
        }: Nep245Withdraw,
    ) -> Result<Promise> {
        self.internal_withdraw(
            sender,
            token_id_amounts
                .iter()
                .cloned()
                .map(|(token_id, amount)| (TokenId::Nep245(token.clone(), token_id), amount.0)),
        )?;

        let (token_ids, amounts) = token_id_amounts.iter().cloned().unzip();
        let mut ext =
            ext_mt_core::ext(token.clone()).with_attached_deposit(NearToken::from_yoctonear(1));
        if let Some(gas) = gas {
            ext = ext.with_static_gas(gas);
        }
        let is_call = msg.is_some();
        Ok(if let Some(msg) = msg {
            ext.mt_batch_transfer_call(receiver_id, token_ids, amounts, None, memo, msg)
        } else {
            ext.mt_batch_transfer(receiver_id, token_ids, amounts, None, memo)
        }
        .then(
            DefuseImpl::ext(CURRENT_ACCOUNT_ID.clone())
                .with_static_gas(DefuseImpl::mt_resolve_withdraw_gas(&token_id_amounts))
                .mt_resolve_withdraw(token, sender_id, token_id_amounts, is_call),
        ))
    }
}

#[near]
impl MultiTokenWithdrawResolver for DefuseImpl {
    #[private]
    fn mt_resolve_withdraw(
        &mut self,
        token: AccountId,
        sender_id: AccountId,
        token_id_amounts: Vec<(nep245::TokenId, U128)>,
        is_call: bool,
    ) -> Vec<U128> {
        let mut used = match env::promise_result(0) {
            PromiseResult::Successful(value) => {
                if is_call {
                    // `mt_batch_transfer_call` returns successfully transferred amounts
                    serde_json::from_slice::<Vec<U128>>(&value)
                        .ok()
                        .filter(|used| used.len() == token_id_amounts.len())
                        .unwrap_or_else(|| vec![U128(0); token_id_amounts.len()])
                } else {
                    // `mt_batch_transfer` returns empty result on success
                    token_id_amounts
                        .iter()
                        .map(|(_token_id, amount)| amount)
                        .copied()
                        .collect()
                }
            }
            PromiseResult::Failed => vec![U128(0); token_id_amounts.len()],
        };

        let account = self.accounts.get_or_create(sender_id);

        for ((token_id, amount), used) in token_id_amounts.into_iter().zip(&mut used) {
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
