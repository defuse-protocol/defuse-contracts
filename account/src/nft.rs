use std::collections::HashMap;

use near_contract_standards::non_fungible_token::{
    core::NonFungibleTokenCore, NonFungibleTokenEnumeration, NonFungibleTokenResolver, Token,
    TokenId,
};
use near_sdk::{json_types::U128, near, AccountId, PromiseOrValue};

use crate::{AccountContractImpl, AccountContractImplExt};

#[near]
impl NonFungibleTokenCore for AccountContractImpl {
    #[payable]
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) {
        self.accounts
            .nft_transfer(receiver_id, token_id, approval_id, memo);
    }

    #[payable]
    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<bool> {
        self.accounts
            .nft_transfer_call(receiver_id, token_id, approval_id, memo, msg)
    }

    fn nft_token(&self, token_id: TokenId) -> Option<Token> {
        self.accounts.nft_token(token_id)
    }
}

#[near]
impl NonFungibleTokenResolver for AccountContractImpl {
    fn nft_resolve_transfer(
        &mut self,
        previous_owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        approved_account_ids: Option<HashMap<AccountId, u64>>,
    ) -> bool {
        self.accounts.nft_resolve_transfer(
            previous_owner_id,
            receiver_id,
            token_id,
            approved_account_ids,
        )
    }
}

#[near]
impl NonFungibleTokenEnumeration for AccountContractImpl {
    fn nft_total_supply(&self) -> U128 {
        self.accounts.nft_total_supply()
    }

    fn nft_tokens(
        &self,
        from_index: Option<U128>, // default: "0"
        limit: Option<u64>,       // default: unlimited (could fail due to gas limit)
    ) -> Vec<Token> {
        self.accounts.nft_tokens(from_index, limit)
    }

    fn nft_supply_for_owner(&self, account_id: AccountId) -> U128 {
        self.accounts.nft_supply_for_owner(account_id)
    }

    fn nft_tokens_for_owner(
        &self,
        account_id: AccountId,
        from_index: Option<U128>, // default: "0"
        limit: Option<u64>,       // default: unlimited (could fail due to gas limit)
    ) -> Vec<Token> {
        self.accounts
            .nft_tokens_for_owner(account_id, from_index, limit)
    }
}
