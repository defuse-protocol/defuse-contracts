use defuse_contracts::multi_ft::{Approval, MultiFungibleToken};
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_contract_standards::fungible_token::{
    FungibleToken, FungibleTokenCore, FungibleTokenResolver,
};
use near_sdk::json_types::U128;
use near_sdk::store::{IterableSet, LookupMap};
use near_sdk::{near, AccountId, PanicOnDefault, PromiseOrValue};

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct MultiFungibleTokenImpl {
    /// Underlying fungible tokens.
    pub tokens: LookupMap<AccountId, FungibleToken>,
    /// List of the fungible token owners.
    pub owners: IterableSet<AccountId>,
    /// Metadata for the multi fungible token (The metadata should be the same for .
    pub metadata: FungibleTokenMetadata,
}

impl MultiFungibleToken for MultiFungibleTokenImpl {
    fn mt_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: AccountId,
        amount: U128,
        approval: Option<Approval>,
        memo: Option<String>,
    ) {
        self.tokens
            .get_mut(&token_id)
            .map(|ft| ft.ft_transfer(receiver_id, amount, memo))
            .unwrap_or_default();
    }

    fn mt_batch_transfer(
        &mut self,
        receiver_id: AccountId,
        token_ids: Vec<AccountId>,
        amounts: Vec<U128>,
        approvals: Option<Vec<Approval>>,
        memo: Option<String>,
    ) {
        token_ids
            .iter()
            .zip(amounts.iter().zip(approvals.unwrap_or_default()))
            .for_each(|(token_id, (amount, approval))| {
                self.mt_transfer(
                    receiver_id.clone(),
                    token_id.clone(),
                    *amount,
                    None,
                    memo.clone(),
                )
            });
    }

    fn mt_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: AccountId,
        amount: U128,
        approval: Option<Vec<Approval>>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        todo!()
    }

    fn mt_batch_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_ids: Vec<AccountId>,
        amounts: Vec<U128>,
        approvals: Option<Vec<Approval>>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<Vec<U128>> {
        todo!()
    }

    fn mt_resolve_transfer(
        &mut self,
        _sender_id: AccountId,
        previous_owner_ids: Vec<AccountId>,
        receiver_id: AccountId,
        token_ids: Vec<AccountId>,
        amounts: Vec<U128>,
        _approvals: Option<Vec<(AccountId, U128, u64)>>,
    ) -> Vec<U128> {
        previous_owner_ids
            .iter()
            .zip(token_ids.iter())
            .zip(amounts.iter())
            .map(|((previous_owner_id, token_id), amount)| {
                self.tokens
                    .get_mut(token_id)
                    .map(|ft| {
                        ft.ft_resolve_transfer(
                            previous_owner_id.clone(),
                            receiver_id.clone(),
                            *amount,
                        )
                    })
                    .unwrap_or_default()
            })
            .collect()
    }

    fn mt_balance_of(&self, account_id: AccountId, token_id: AccountId) -> U128 {
        self.tokens
            .get(&token_id)
            .map(|ft| ft.ft_balance_of(account_id))
            .unwrap_or_default()
    }

    fn mt_batch_balance_of(&self, account_id: AccountId, token_ids: Vec<AccountId>) -> Vec<U128> {
        token_ids
            .iter()
            .map(|id| self.mt_balance_of(account_id.clone(), id.clone()))
            .collect()
    }

    fn mt_supply(&self, token_id: AccountId) -> Option<U128> {
        self.tokens.get(&token_id).map(|ft| ft.ft_total_supply())
    }

    fn mt_batch_supply(&self, token_ids: Vec<AccountId>) -> Vec<Option<U128>> {
        token_ids
            .iter()
            .map(|id| self.mt_supply(id.clone()))
            .collect()
    }
}
