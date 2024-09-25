use defuse_contracts::{
    defuse::tokens::TokenId,
    nep245::{self, MultiTokenCore},
    utils::cache::PREDECESSOR_ACCOUNT_ID,
};
use near_sdk::{json_types::U128, near, require, AccountId, PromiseOrValue};

use crate::{DefuseImpl, DefuseImplExt};

#[near]
impl MultiTokenCore for DefuseImpl {
    fn mt_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: nep245::TokenId,
        amount: U128,
        approval: Option<(AccountId, u64)>,
        memo: Option<String>,
    ) {
        self.internal_transfer(
            &PREDECESSOR_ACCOUNT_ID,
            &receiver_id,
            token_id.parse().unwrap(),
            amount.0,
        )
        .unwrap()
    }

    fn mt_batch_transfer(
        &mut self,
        receiver_id: AccountId,
        token_ids: Vec<nep245::TokenId>,
        amounts: Vec<U128>,
        approvals: Option<Vec<Option<(AccountId, u64)>>>,
        memo: Option<String>,
    ) {
        require!(
            token_ids.len() == amounts.len(),
            "token_ids should be the same length as amounts"
        );

        let sender_id = &PREDECESSOR_ACCOUNT_ID;
        for (token_id, amount) in token_ids.into_iter().zip(amounts) {
            self.internal_transfer(sender_id, &receiver_id, token_id.parse().unwrap(), amount.0)
                .unwrap()
        }
    }

    fn mt_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: nep245::TokenId,
        amount: U128,
        approval: Option<(AccountId, u64)>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        todo!()
    }

    fn mt_batch_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_ids: Vec<nep245::TokenId>,
        amounts: Vec<U128>,
        approvals: Option<Vec<Option<(AccountId, u64)>>>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<Vec<U128>> {
        todo!()
    }

    fn mt_balance_of(&self, account_id: AccountId, token_id: nep245::TokenId) -> U128 {
        U128(self.internal_mt_balance_of(&account_id, &token_id.parse().unwrap()))
    }

    fn mt_batch_balance_of(
        &self,
        account_id: AccountId,
        token_ids: Vec<nep245::TokenId>,
    ) -> Vec<U128> {
        token_ids
            .into_iter()
            .map(|token_id| token_id.parse().unwrap())
            .map(|token_id| self.internal_mt_balance_of(&account_id, &token_id))
            .map(U128)
            .collect()
    }

    fn mt_supply(&self, token_id: nep245::TokenId) -> Option<U128> {
        todo!()
    }

    fn mt_batch_supply(&self, token_ids: Vec<nep245::TokenId>) -> Vec<Option<U128>> {
        todo!()
    }
}

impl DefuseImpl {
    fn internal_mt_balance_of(&self, account_id: &AccountId, token_id: &TokenId) -> u128 {
        self.accounts
            .get(account_id)
            .map(|account| account.token_balances.balance_of(token_id))
            .unwrap_or_default()
    }
}
