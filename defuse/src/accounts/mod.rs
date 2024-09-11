mod account;
mod named;
mod state;
mod typed;

use defuse_contracts::utils::prefix::NestPrefix;
use near_account_id::AccountType;
use near_sdk::{near, store::IterableMap, AccountId, BorshStorageKey, IntoStorageKey};

pub use self::{account::*, named::*, state::*, typed::*};

#[derive(Debug)]
#[near(serializers = [borsh])]
pub struct Accounts {
    accounts: IterableMap<AccountId, TypedAccount>,
    prefix: Vec<u8>,
}

impl Accounts {
    #[inline]
    pub fn new<S>(prefix: S) -> Self
    where
        S: IntoStorageKey,
    {
        let prefix = prefix.into_storage_key();
        Self {
            accounts: IterableMap::new(prefix.as_slice().nest(AccountsPrefix::Accounts)),
            prefix,
        }
    }

    #[inline]
    pub fn get(&self, account_id: &AccountId) -> Option<&TypedAccount> {
        self.accounts.get(account_id)
    }

    #[inline]
    pub fn get_mut(&mut self, account_id: &AccountId) -> Option<&mut TypedAccount> {
        self.accounts.get_mut(account_id)
    }

    #[inline]
    pub fn get_or_insert(&mut self, account_id: AccountId) -> &mut TypedAccount {
        self.accounts
            .entry(account_id)
            .or_insert_with_key(|account_id| {
                let prefix = self
                    .prefix
                    .as_slice()
                    .nest(AccountsPrefix::Account)
                    .nest(account_id.as_bytes());

                match account_id.get_account_type() {
                    AccountType::NamedAccount => TypedAccount::Named(NamedAccount::new(prefix)),
                    AccountType::NearImplicitAccount | AccountType::EthImplicitAccount => {
                        TypedAccount::Implicit(Account::new(prefix))
                    }
                }
            })
    }
}

#[derive(BorshStorageKey)]
#[near(serializers = [borsh])]
enum AccountsPrefix {
    Accounts,
    Account,
}
