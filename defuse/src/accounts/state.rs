use defuse_contracts::utils::prefix::NestPrefix;
use near_sdk::{near, BorshStorageKey, IntoStorageKey};

use crate::tokens::TokensBalances;

#[derive(Debug)]
#[near(serializers = [borsh])]
pub struct AccountState {
    pub token_balances: TokensBalances,
}

impl AccountState {
    pub fn new<S>(prefix: S) -> Self
    where
        S: IntoStorageKey,
    {
        let parent = prefix.into_storage_key();

        Self {
            token_balances: TokensBalances::new(
                parent.as_slice().nest(AccountStatePrefix::TokenBalances),
            ),
        }
    }
}

#[derive(BorshStorageKey)]
#[near(serializers = [borsh])]
enum AccountStatePrefix {
    TokenBalances,
}
