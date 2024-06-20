use std::collections::HashMap;

use defuse_contracts::account::{Account, AccountContract};
use near_contract_standards::non_fungible_token::{
    core::NonFungibleTokenCore, NonFungibleToken, NonFungibleTokenEnumeration,
    NonFungibleTokenResolver, Token, TokenId,
};
use near_sdk::{
    env, json_types::U128, near, store::LookupSet, AccountId, BorshStorageKey, PanicOnDefault,
    PromiseOrValue,
};

use self::account_db::AccountDb;

mod account_db;

#[derive(BorshStorageKey)]
#[near(serializers=[borsh])]
enum Prefix {
    OwnerById,
    Metatada,
    Enumeration,
    Approvals,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct AccountContractImpl {
    accounts: NonFungibleToken,
    /// MPC contract id.
    mpc_contract_id: AccountId,
    // /// List of indexers. Accounts which allow to add a new account.
    // indexers: LookupSet<AccountId>,
}

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
            .nft_transfer(receiver_id, token_id, approval_id, memo)
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

#[near]
impl AccountContract for AccountContractImpl {
    #[payable]
    // TODO: make owner optional and default to env::predecessor_account_id()?
    fn create_account(&mut self, owner: AccountId, derivation_path: String) {
        // // Only indexers can call this transaction.
        // let predecessor_id = env::predecessor_account_id();
        // self.assert_indexer(&predecessor_id);

        self.accounts.internal_mint(derivation_path, owner, None);
    }

    fn mpc_contract(&self) -> &AccountId {
        &self.mpc_contract_id
    }
}

#[near]
impl AccountContractImpl {
    #[init]
    #[must_use]
    #[allow(clippy::use_self)]
    pub fn new(owner: AccountId, mpc_contract_id: AccountId) -> Self {
        Self {
            accounts: NonFungibleToken::new::<_, u8, u8, u8>(
                Prefix::OwnerById,
                owner,
                None,
                None,
                None,
            ),
            mpc_contract_id,
        }
    }

    #[private]
    pub fn set_mpc_contract(&mut self, contract_id: AccountId) {
        self.mpc_contract_id = contract_id;
    }

    // fn assert_indexer(&self, account_id: &AccountId) {
    //     assert!(
    //         self.indexers.contains(account_id),
    //         "Only indexers allow adding an account"
    //     );
    // }
}
