use defuse_contracts::{crypto::PublicKey, utils::prefix::NestPrefix};
use impl_tools::autoimpl;
use near_sdk::{near, store::IterableSet, BorshStorageKey, IntoStorageKey};

use super::Account;

#[derive(Debug)]
#[near(serializers = [borsh])]
#[autoimpl(Deref using self.account)]
#[autoimpl(DerefMut using self.account)]

pub struct NamedAccount {
    public_keys: IterableSet<PublicKey>,
    account: Account,
}

impl NamedAccount {
    #[inline]
    pub fn new<S>(prefix: S) -> Self
    where
        S: IntoStorageKey,
    {
        let parent = prefix.into_storage_key();

        #[derive(BorshStorageKey)]
        #[near(serializers = [borsh])]
        enum NamedAccountPrefix {
            PublicKeys,
            Account,
        }

        Self {
            public_keys: IterableSet::new(parent.as_slice().nest(NamedAccountPrefix::PublicKeys)),
            account: Account::new(parent.nest(NamedAccountPrefix::Account)),
        }
    }

    #[inline]
    pub fn has_public_key(&self, public_key: &PublicKey) -> bool {
        self.public_keys.contains(public_key)
    }

    #[inline]
    pub fn iter_public_keys(&self) -> impl Iterator<Item = &'_ PublicKey> {
        self.public_keys.iter()
    }

    #[inline]
    pub fn add_public_key(&mut self, public_key: PublicKey) -> bool {
        self.public_keys.insert(public_key)
    }

    #[inline]
    pub fn remove_public_key(&mut self, public_key: &PublicKey) -> bool {
        self.public_keys.remove(public_key)
    }
}
