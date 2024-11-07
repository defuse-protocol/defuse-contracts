use std::borrow::Cow;

use defuse_contracts::{
    crypto::PublicKey,
    defuse::{
        accounts::PublicKeyEvent,
        events::{DefuseEvent, DefuseIntentEmit},
        intents::AccountEvent,
        DefuseError, Result,
    },
    nep413::U256,
    utils::prefix::NestPrefix,
};
use impl_tools::autoimpl;
use near_account_id::AccountType;
use near_sdk::{
    borsh::BorshSerialize, near, store::IterableSet, AccountId, BorshStorageKey, IntoStorageKey,
};

use super::{AccountState, Nonces};

#[derive(Debug)]
#[near(serializers = [borsh])]
#[autoimpl(Deref using self.state)]
#[autoimpl(DerefMut using self.state)]
pub struct Account {
    nonces: Nonces,

    implicit_public_key_removed: bool,
    public_keys: IterableSet<PublicKey>,

    pub state: AccountState,

    prefix: Vec<u8>,
}

impl Account {
    #[inline]
    pub fn new<S>(prefix: S, me: &AccountId) -> Self
    where
        S: IntoStorageKey,
    {
        let prefix = prefix.into_storage_key();

        Self {
            nonces: Nonces::new(prefix.as_slice().nest(AccountPrefix::Nonces)),
            implicit_public_key_removed: matches!(me.get_account_type(), AccountType::NamedAccount),
            public_keys: IterableSet::new(prefix.as_slice().nest(AccountPrefix::PublicKeys)),
            state: AccountState::new(prefix.as_slice().nest(AccountPrefix::State)),
            prefix,
        }
    }

    #[inline]
    pub fn add_public_key(&mut self, me: &AccountId, public_key: PublicKey) -> Result<()> {
        DefuseEvent::PublicKeyAdded(AccountEvent::new(
            Cow::Borrowed(me.as_ref()),
            PublicKeyEvent {
                public_key: Cow::Borrowed(&public_key),
            },
        ))
        .emit();

        if !self.maybe_add_public_key(me, public_key) {
            return Err(DefuseError::PublicKeyExists);
        }
        Ok(())
    }

    #[inline]
    fn maybe_add_public_key(&mut self, me: &AccountId, public_key: PublicKey) -> bool {
        if me == &public_key.to_implicit_account_id() {
            let was_removed = self.implicit_public_key_removed;
            self.implicit_public_key_removed = false;
            was_removed
        } else {
            self.public_keys.insert(public_key)
        }
    }

    #[inline]
    pub fn remove_public_key(&mut self, me: &AccountId, public_key: &PublicKey) -> Result<()> {
        DefuseEvent::PublicKeyRemoved(AccountEvent::new(
            Cow::Borrowed(me.as_ref()),
            PublicKeyEvent {
                public_key: Cow::Borrowed(public_key),
            },
        ))
        .emit();

        if !self.maybe_remove_public_key(me, public_key) {
            return Err(DefuseError::PublicKeyNotExist);
        }
        Ok(())
    }

    #[inline]
    fn maybe_remove_public_key(&mut self, me: &AccountId, public_key: &PublicKey) -> bool {
        if me == &public_key.to_implicit_account_id() {
            let was_removed = self.implicit_public_key_removed;
            self.implicit_public_key_removed = true;
            !was_removed
        } else {
            self.public_keys.remove(public_key)
        }
    }

    #[inline]
    pub fn has_public_key(&self, me: &AccountId, public_key: &PublicKey) -> bool {
        !self.implicit_public_key_removed && me == &public_key.to_implicit_account_id()
            || self.public_keys.contains(public_key)
    }

    #[inline]
    pub fn iter_public_keys(&self, me: &AccountId) -> impl Iterator<Item = PublicKey> + '_ {
        self.public_keys.iter().cloned().chain(
            (!self.implicit_public_key_removed)
                .then(|| PublicKey::from_implicit_account_id(me))
                .flatten(),
        )
    }

    #[inline]
    pub fn is_nonce_used(&self, nonce: U256) -> bool {
        self.nonces.is_used(nonce)
    }

    #[inline]
    pub fn commit_nonce(&mut self, n: U256) -> Result<()> {
        self.nonces.commit(n)
    }
}

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "::near_sdk::borsh")]
enum AccountPrefix {
    Nonces,
    PublicKeys,
    State,
}
