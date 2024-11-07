use defuse_contracts::{
    nep245::{
        resolver::MultiTokenResolver, ClearedApproval, MtEventEmit, MtTransferEvent, TokenId,
    },
    utils::{UnwrapOrPanic, UnwrapOrPanicError},
};
use near_sdk::{env, json_types::U128, near, require, serde_json, AccountId, Gas, PromiseResult};

use crate::{DefuseImpl, DefuseImplExt};

impl DefuseImpl {
    // TODO: more accurate numbers
    const MT_RESOLVE_TRANSFER_GAS_BASE: Gas = Gas::from_tgas(5);
    const MT_RESOLVE_TRANSFER_GAS_PER_TOKEN_ID: Gas = Gas::from_tgas(1);

    pub(super) const fn mt_resolve_transfer_gas(token_count: u64) -> Gas {
        // if these conversions overflow, then
        // it should have exceeded gas before
        Self::MT_RESOLVE_TRANSFER_GAS_BASE
            .saturating_add(Self::MT_RESOLVE_TRANSFER_GAS_PER_TOKEN_ID.saturating_mul(token_count))
    }
}

#[near]
impl MultiTokenResolver for DefuseImpl {
    #[private]
    fn mt_resolve_transfer(
        &mut self,
        previous_owner_ids: Vec<AccountId>,
        receiver_id: AccountId,
        token_ids: Vec<TokenId>,
        #[allow(unused_mut)] mut amounts: Vec<U128>,
        approvals: Option<Vec<Option<Vec<ClearedApproval>>>>,
    ) -> Vec<U128> {
        require!(approvals.is_none(), "approvals are not supported");
        require!(!amounts.is_empty(), "zero amounts");
        require!(
            previous_owner_ids.len() == token_ids.len(),
            "previous_owner_ids.len() != token_ids.len()"
        );
        require!(
            token_ids.len() == amounts.len(),
            "token_ids.len() != amounts.len()"
        );

        let mut refunds = match env::promise_result(0) {
            PromiseResult::Successful(value) => serde_json::from_slice::<Vec<U128>>(&value)
                .ok()
                .filter(|refund| refund.len() == amounts.len())
                .unwrap_or_else(|| amounts.clone()),
            PromiseResult::Failed => amounts.clone(),
        };

        let sender_id = previous_owner_ids.first().cloned().unwrap_or_panic();

        for ((token_id, previous_owner_id), (amount, refund)) in token_ids
            .iter()
            .map(|token_id| token_id.parse().unwrap_or_panic_display())
            .zip(previous_owner_ids)
            .zip(amounts.iter_mut().zip(&mut refunds))
        {
            require!(
                sender_id == previous_owner_id,
                "approvals are not supported"
            );

            refund.0 = refund.0.min(amount.0);
            if refund.0 == 0 {
                // no need to refund anything
                continue;
            }

            let Some(receiver) = self.accounts.get_mut(&receiver_id) else {
                // receiver doesn't have an account anymore
                break;
            };
            let receiver_balance = receiver.token_balances.balance_of(&token_id);
            if receiver_balance == 0 {
                // receiver doesn't have any balance anymore
                continue;
            }
            // refund maximum what we can
            refund.0 = refund.0.min(receiver_balance);
            // withdraw refund
            receiver
                .token_balances
                .withdraw(token_id.clone(), refund.0)
                .unwrap_or_panic();

            // deposit refund
            let previous_owner = self.accounts.get_or_create(previous_owner_id);
            previous_owner
                .token_balances
                .deposit(token_id, refund.0)
                .unwrap_or_panic();

            // update as used amount in-place
            amount.0 -= refund.0;
        }

        [MtTransferEvent {
            authorized_id: None,
            old_owner_id: receiver_id.into(),
            new_owner_id: sender_id.into(),
            token_ids: token_ids.into(),
            amounts: refunds.into(),
            memo: Some("refund".into()),
        }]
        .emit();

        amounts
    }
}
