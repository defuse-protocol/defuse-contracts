use defuse_contracts::{
    defuse::{
        tokens::{
            nep245::{MultiTokenWithdrawResolver, MultiTokenWithdrawer},
            TokenId,
        },
        DefuseError, Result,
    },
    nep245::{self, ext_mt_core},
    utils::cache::{CURRENT_ACCOUNT_ID, PREDECESSOR_ACCOUNT_ID},
};
use near_sdk::{
    assert_one_yocto, env, json_types::U128, near, require, serde_json, AccountId, NearToken,
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
        self.internal_mt_withdraw(token, receiver_id, token_ids, amounts, memo, msg)
            .unwrap()
    }
}

impl DefuseImpl {
    fn internal_mt_withdraw(
        &mut self,
        token: AccountId,
        receiver_id: AccountId,
        token_ids: Vec<nep245::TokenId>,
        amounts: Vec<U128>,
        memo: Option<String>,
        msg: Option<String>,
    ) -> Result<PromiseOrValue<Vec<U128>>> {
        // TODO: maybe do not panic but return an error?
        require!(
            token_ids.len() == amounts.len(),
            "token_ids should be the same length as amounts"
        );

        let account = self
            .accounts
            .get_mut(&PREDECESSOR_ACCOUNT_ID)
            .ok_or(DefuseError::AccountNotFound)?;
        for (token_id, amount) in token_ids.iter().zip(&amounts) {
            account
                .token_balances
                .withdraw(&TokenId::Nep245(token.clone(), token_id.clone()), amount.0)?;
        }

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
                // TODO: with static gas
                .mt_resolve_withdraw(
                    token,
                    PREDECESSOR_ACCOUNT_ID.clone(),
                    token_ids,
                    amounts,
                    is_call,
                ),
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
        is_call: bool,
    ) -> Vec<U128> {
        let mut used = match env::promise_result(0) {
            PromiseResult::Successful(value) => {
                if is_call {
                    // `mt_batch_transfer_call` returns successfully transferred amounts
                    if let Ok(used) = serde_json::from_slice::<Vec<U128>>(&value) {
                        require!(used.len() == amounts.len(), "invalid response length");
                        used
                    } else {
                        vec![U128(0); amounts.len()]
                    }
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
                // Are we sure that we want to ignore that?
                let _ = account
                    .token_balances
                    .deposit(TokenId::Nep245(token.clone(), token_id), refund);
            }
        }

        used
    }
}
