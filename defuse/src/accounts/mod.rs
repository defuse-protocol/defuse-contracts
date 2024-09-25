mod account;
mod nonces;
mod state;

use std::collections::HashSet;

pub use self::{account::*, nonces::*, state::*};

use defuse_contracts::{
    crypto::PublicKey,
    defuse::accounts::AccountManager,
    nep413::Nonce,
    utils::{cache::PREDECESSOR_ACCOUNT_ID, prefix::NestPrefix, serde::wrappers::DisplayFromStr},
};
use near_sdk::{
    borsh::BorshSerialize,
    near,
    store::{iterable_map::Entry, IterableMap},
    AccountId, BorshStorageKey, IntoStorageKey,
};

use crate::{DefuseImpl, DefuseImplExt};

#[near]
impl AccountManager for DefuseImpl {
    fn public_keys_of(&self, account_id: &AccountId) -> HashSet<PublicKey> {
        self.accounts
            .get(account_id)
            .into_iter()
            .flat_map(Account::iter_public_keys)
            .copied()
            .collect()
    }

    fn add_public_key(&mut self, public_key: PublicKey) {
        self.accounts
            .get_or_create(PREDECESSOR_ACCOUNT_ID.clone())
            .add_public_key(public_key);
    }

    fn remove_public_key(&mut self, public_key: &PublicKey) -> bool {
        self.accounts
            .get_mut(&PREDECESSOR_ACCOUNT_ID)
            .map_or(false, |account| account.remove_public_key(public_key))
    }

    fn next_nonce_available(
        &self,
        account_id: &AccountId,
        public_key: &PublicKey,
        start: Option<DisplayFromStr<Nonce>>,
    ) -> Option<DisplayFromStr<Nonce>> {
        self.accounts
            .get(account_id)
            .and_then(move |account| account.public_key_nonces(public_key))
            .and_then(move |nonces| nonces.next_unused(start.map(|n| n.0)))
            .map(DisplayFromStr)
    }
}

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
