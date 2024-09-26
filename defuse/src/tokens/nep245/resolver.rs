use defuse_contracts::nep245::{resolver::MultiTokenResolver, ClearedApproval, TokenId};
use near_sdk::{env, json_types::U128, near, require, serde_json, AccountId, PromiseResult};

use crate::{DefuseImpl, DefuseImplExt};

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
        require!(
            previous_owner_ids.len() == token_ids.len(),
            "previous_owner_ids should be the same length as tokens_ids"
        );
        require!(
            token_ids.len() == amounts.len(),
            "token_ids should be the same length as amounts"
        );
        require!(approvals.is_none(), "approvals are not supported");

        let refund = match env::promise_result(0) {
            PromiseResult::Successful(value) => {
                if let Ok(refund) = serde_json::from_slice::<Vec<U128>>(&value) {
                    require!(refund.len() == amounts.len(), "invalid response length");
                    refund
                } else {
                    amounts.clone()
                }
            }
            PromiseResult::Failed => amounts.clone(),
        };

        for ((token_id, previous_owner_id), (amount, refund)) in token_ids
            .into_iter()
            .zip(previous_owner_ids)
            .zip(amounts.iter_mut().zip(refund))
        {
            let mut refund = refund.0.min(amount.0);
            if refund > 0 {
                let token_id = token_id.parse().unwrap();

                if let Some(receiver) = self.accounts.get_mut(&receiver_id) {
                    let receiver_balance = receiver.token_balances.balance_of(&token_id);
                    if receiver_balance > 0 {
                        // refund maximum what we can
                        refund = refund.min(receiver_balance);
                        // withdraw refund
                        receiver
                            .token_balances
                            .withdraw(token_id.clone(), refund)
                            .unwrap();

                        // deposit refund
                        let previous_owner = self.accounts.get_or_create(previous_owner_id);
                        previous_owner
                            .token_balances
                            .deposit(token_id, refund)
                            .unwrap();

                        // update as used amount in-place
                        amount.0 -= refund;

                        // TODO: log refund
                    }
                }
            }
        }

        amounts
    }
}
