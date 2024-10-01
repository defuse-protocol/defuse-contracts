use defuse_contracts::{
    crypto::{Payload, PublicKey, SignedPayload},
    defuse::{payload::ValidatePayloadAs, DefuseError, Result},
    nep413::Nep413Payload,
    utils::{cache::CURRENT_ACCOUNT_ID, prefix::NestPrefix, Lock},
};
use impl_tools::autoimpl;
use near_account_id::AccountType;
use near_sdk::{
    borsh::BorshSerialize, near, store::IterableMap, AccountId, BorshStorageKey, IntoStorageKey,
};

use super::{AccountState, Nonces};

#[derive(Debug)]
#[near(serializers = [borsh])]
#[autoimpl(Deref using self.state)]
#[autoimpl(DerefMut using self.state)]
pub struct Account {
    /// Nonces used in case of implicit [`AccountId`]
    implicit_nonces: Lock<Nonces>,
    public_keys: IterableMap<PublicKey, Lock<Nonces>>,

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
            implicit_nonces: Lock::new(
                Nonces::new(prefix.as_slice().nest(AccountPrefix::ImplicitNonces)),
                matches!(me.get_account_type(), AccountType::NamedAccount),
            ),
            public_keys: IterableMap::new(prefix.as_slice().nest(AccountPrefix::PublicKeys)),
            state: AccountState::new(prefix.as_slice().nest(AccountPrefix::State)),
            prefix,
        }
    }

    #[inline]
    pub fn add_public_key(&mut self, me: &AccountId, public_key: PublicKey) {
        if me == &public_key.to_implicit_account_id() {
            &mut self.implicit_nonces
        } else {
            self.public_keys
                .entry(public_key)
                .or_insert_with_key(|public_key| {
                    Nonces::new(
                        self.prefix
                            .as_slice()
                            .nest(AccountPrefix::PublicKeyNonces(public_key)),
                    )
                    .into()
                })
        }
        .force_unlock();
    }

    #[inline]
    pub fn deactivate_public_key(&mut self, me: &AccountId, public_key: &PublicKey) {
        if me == &public_key.to_implicit_account_id() {
            Some(&mut self.implicit_nonces)
        } else {
            self.public_keys.get_mut(public_key)
        }
        .map(Lock::force_lock);
    }

    #[inline]
    pub fn is_public_key_active(&self, me: &AccountId, public_key: &PublicKey) -> bool {
        self.public_key_nonces(me, public_key).is_some()
    }

    #[inline]
    pub fn iter_active_public_keys(&self) -> impl Iterator<Item = &'_ PublicKey> {
        self.public_keys
            .iter()
            .filter_map(|(public_key, nonces)| nonces.is_unlocked().then_some(public_key))
    }

    #[must_use]
    #[inline]
    pub fn public_key_nonces(&self, me: &AccountId, public_key: &PublicKey) -> Option<&Nonces> {
        if me == &public_key.to_implicit_account_id() {
            Some(&self.implicit_nonces)
        } else {
            self.public_keys.get(public_key)
        }
        .and_then(Lock::as_unlocked)
    }

    #[must_use]
    #[inline]
    pub fn public_key_nonces_mut(
        &mut self,
        me: &AccountId,
        public_key: &PublicKey,
    ) -> Option<&mut Nonces> {
        if me == &public_key.to_implicit_account_id() {
            Some(&mut self.implicit_nonces)
        } else {
            self.public_keys.get_mut(public_key)
        }
        .and_then(Lock::as_unlocked_mut)
    }

    pub fn verify_signed_as_nep413<S, T>(
        &mut self,
        me: &AccountId,
        signed: SignedPayload<S>,
    ) -> Result<T>
    where
        S: Payload + ValidatePayloadAs<Nep413Payload<T>, Error: Into<DefuseError>>,
    {
        // verify signature and derive its public key
        let public_key = signed.verify().ok_or(DefuseError::InvalidSignature)?;

        // extract NEP-413 payload
        let payload: Nep413Payload<T> = signed.payload.validate_as().map_err(Into::into)?;

        // check recipient
        if payload.recipient != *CURRENT_ACCOUNT_ID {
            return Err(DefuseError::WrongRecipient);
        }

        // commit nonce for public key
        self.public_key_nonces_mut(me, &public_key)
            .ok_or(DefuseError::InvalidSignature)?
            .commit(payload.nonce)?;

        // return inner message
        Ok(payload.message)
    }
}

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "::near_sdk::borsh")]
enum AccountPrefix<'a> {
    ImplicitNonces,
    PublicKeys,
    PublicKeyNonces(&'a PublicKey),
    State,
}
