use defuse_contracts::{
    defuse::tokens::TokenId,
    nep245::{self, receiver::ext_mt_receiver, MultiTokenCore},
    utils::cache::{CURRENT_ACCOUNT_ID, PREDECESSOR_ACCOUNT_ID},
};
use near_sdk::{assert_one_yocto, json_types::U128, near, require, AccountId, PromiseOrValue};

use crate::{DefuseImpl, DefuseImplExt};

#[near]
impl MultiTokenCore for DefuseImpl {
    #[payable]
    fn mt_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: nep245::TokenId,
        amount: U128,
        approval: Option<(AccountId, u64)>,
        memo: Option<String>,
    ) {
        self.mt_batch_transfer(
            receiver_id,
            [token_id].into(),
            [amount].into(),
            approval.map(|a| vec![Some(a)]),
            memo,
        );
    }

    #[payable]
    fn mt_batch_transfer(
        &mut self,
        receiver_id: AccountId,
        token_ids: Vec<nep245::TokenId>,
        amounts: Vec<U128>,
        approvals: Option<Vec<Option<(AccountId, u64)>>>,
        memo: Option<String>,
    ) {
        assert_one_yocto();
        require!(
            token_ids.len() == amounts.len(),
            "token_ids should be the same length as amounts"
        );
        require!(approvals.is_none(), "approvals are not supported");

        self.internal_transfer(
            &PREDECESSOR_ACCOUNT_ID,
            receiver_id,
            token_ids
                .into_iter()
                .map(|token_id| token_id.parse().unwrap())
                .zip(amounts.into_iter().map(|a| a.0))
                .collect(),
            memo,
        )
        .unwrap()
    }

    #[payable]
    fn mt_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: nep245::TokenId,
        amount: U128,
        approval: Option<(AccountId, u64)>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<Vec<U128>> {
        self.mt_batch_transfer_call(
            receiver_id,
            [token_id].into(),
            [amount].into(),
            approval.map(|a| vec![Some(a)]),
            memo,
            msg,
        )
    }

    #[payable]
    fn mt_batch_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_ids: Vec<nep245::TokenId>,
        amounts: Vec<U128>,
        approvals: Option<Vec<Option<(AccountId, u64)>>>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<Vec<U128>> {
        assert_one_yocto();
        require!(
            token_ids.len() == amounts.len(),
            "token_ids should be the same length as amounts"
        );
        require!(approvals.is_none(), "approvals are not supported");

        self.internal_transfer(
            &PREDECESSOR_ACCOUNT_ID,
            receiver_id.clone(),
            token_ids
                .iter()
                .map(|token_id| token_id.parse().unwrap())
                .zip(amounts.iter().map(|a| a.0))
                .collect(),
            memo,
        )
        .unwrap();

        let previous_owner_ids = vec![PREDECESSOR_ACCOUNT_ID.clone(); token_ids.len()];

        ext_mt_receiver::ext(receiver_id.clone())
            .mt_on_transfer(
                PREDECESSOR_ACCOUNT_ID.clone(),
                previous_owner_ids.clone(),
                token_ids.clone(),
                amounts.clone(),
                msg,
            )
            .then(
                Self::ext(CURRENT_ACCOUNT_ID.clone())
                    // TODO: with static gas
                    .mt_resolve_transfer(previous_owner_ids, receiver_id, token_ids, amounts, None),
            )
            .into()
    }

    // TODO: mt_token

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
        Some(U128(
            self.total_supplies.balance_of(&token_id.parse().ok()?),
        ))
    }

    fn mt_batch_supply(&self, token_ids: Vec<nep245::TokenId>) -> Vec<Option<U128>> {
        token_ids
            .into_iter()
            .map(|token_id| self.mt_supply(token_id))
            .collect()
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
