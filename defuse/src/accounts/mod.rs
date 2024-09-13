mod account;
mod nonces;
mod state;

use defuse_contracts::utils::prefix::NestPrefix;
use near_sdk::{
    borsh::BorshSerialize,
    near,
    store::{iterable_map::Entry, IterableMap},
    AccountId, BorshStorageKey, IntoStorageKey,
};

pub use self::{account::*, nonces::*, state::*};

#[derive(Debug)]
#[near(serializers = [borsh])]
pub struct Accounts {
    accounts: IterableMap<AccountId, Account>,
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
    pub fn get(&self, account_id: &AccountId) -> Option<&Account> {
        self.accounts.get(account_id)
    }

    #[inline]
    pub fn get_mut(&mut self, account_id: &AccountId) -> Option<&mut Account> {
        self.accounts.get_mut(account_id)
    }

    #[inline]
    pub fn get_or_create(&mut self, account_id: AccountId) -> &mut Account {
        self.get_or_create_fresh(account_id).into_account()
    }

    #[inline]
    pub fn get_or_create_fresh(&mut self, account_id: AccountId) -> MaybeFreshAccount<'_> {
        match self.accounts.entry(account_id) {
            Entry::Occupied(account) => MaybeFreshAccount::new(account.into_mut(), false),
            Entry::Vacant(entry) => {
                let account = Account::new(
                    self.prefix
                        .as_slice()
                        .nest(AccountsPrefix::Account(entry.key())),
                );
                MaybeFreshAccount::new(entry.insert(account), true)
            }
        }
    }
}

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "::near_sdk::borsh")]
enum AccountsPrefix<'a> {
    Accounts,
    Account(&'a AccountId),
}
