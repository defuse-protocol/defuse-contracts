use defuse_core::tokens::{TokenAmounts, TokenId};
use defuse_near_utils::NestPrefix;
use near_sdk::{near, store::IterableMap, BorshStorageKey, IntoStorageKey};

#[derive(Debug)]
#[near(serializers = [borsh])]
pub struct AccountState {
    pub token_balances: TokenAmounts<IterableMap<TokenId, u128>>,
}

impl AccountState {
    pub fn new<S>(prefix: S) -> Self
    where
        S: IntoStorageKey,
    {
        let parent = prefix.into_storage_key();

        Self {
            token_balances: TokenAmounts::new(IterableMap::new(
                parent.as_slice().nest(AccountStatePrefix::TokenBalances),
            )),
        }
    }
}

#[derive(BorshStorageKey)]
#[near(serializers = [borsh])]
enum AccountStatePrefix {
    TokenBalances,
}
