mod account;
mod nonces;
mod state;

pub use self::{account::*, nonces::*, state::*};

use std::collections::HashSet;

use defuse_contracts::{
    crypto::PublicKey,
    defuse::accounts::AccountManager,
    nep413::U256,
    utils::{
        cache::PREDECESSOR_ACCOUNT_ID, prefix::NestPrefix, serde::wrappers::Base64, UnwrapOrPanic,
    },
};

use near_sdk::{
    assert_one_yocto, borsh::BorshSerialize, near, store::IterableMap, AccountId, BorshStorageKey,
    IntoStorageKey,
};

use crate::{DefuseImpl, DefuseImplExt};

#[near]
impl AccountManager for DefuseImpl {
    fn has_public_key(&self, account_id: &AccountId, public_key: &PublicKey) -> bool {
        self.accounts
            .get(account_id)
            .map(|account| account.has_public_key(account_id, public_key))
            .unwrap_or_else(|| account_id == &public_key.to_implicit_account_id())
    }

    fn public_keys_of(&self, account_id: &AccountId) -> HashSet<PublicKey> {
        self.accounts
            .get(account_id)
            .map(|account| account.iter_public_keys(account_id).collect())
            .unwrap_or_else(|| {
                PublicKey::from_implicit_account_id(account_id)
                    .into_iter()
                    .collect()
            })
    }

    #[payable]
    fn add_public_key(&mut self, public_key: PublicKey) {
        assert_one_yocto();
        self.accounts
            .get_or_create(PREDECESSOR_ACCOUNT_ID.clone())
            .add_public_key(&PREDECESSOR_ACCOUNT_ID, public_key)
            .unwrap_or_panic()
    }

    #[payable]
    fn remove_public_key(&mut self, public_key: &PublicKey) {
        assert_one_yocto();
        self.accounts
            // create account if doesn't exist, so the user can opt out of implicit public key
            .get_or_create(PREDECESSOR_ACCOUNT_ID.clone())
            .remove_public_key(&PREDECESSOR_ACCOUNT_ID, public_key)
            .unwrap_or_panic()
    }

    fn is_nonce_used(&self, account_id: &AccountId, nonce: Base64<U256>) -> bool {
        self.accounts
            .get(account_id)
            .map(move |account| account.is_nonce_used(nonce.into_inner()))
            .unwrap_or_default()
    }

    #[payable]
    fn invalidate_nonces(&mut self, nonces: Vec<Base64<U256>>) {
        assert_one_yocto();
        let account = self.accounts.get_or_create(PREDECESSOR_ACCOUNT_ID.clone());
        for n in nonces.into_iter().map(Base64::into_inner) {
            account.commit_nonce(n).unwrap_or_panic();
        }
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
        self.accounts
            .entry(account_id)
            .or_insert_with_key(|account_id| {
                Account::new(
                    self.prefix
                        .as_slice()
                        .nest(AccountsPrefix::Account(account_id)),
                    account_id,
                )
            })
    }
}

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "::near_sdk::borsh")]
enum AccountsPrefix<'a> {
    Accounts,
    Account(&'a AccountId),
}
