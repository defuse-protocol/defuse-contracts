use defuse_contracts::{
    crypto::{Payload, PublicKey, SignedPayload},
    defuse::{payload::ValidatePayloadAs, DefuseError},
    nep413::Nep413Payload,
    utils::{cache::CURRENT_ACCOUNT_ID, prefix::NestPrefix},
};
use impl_tools::autoimpl;
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
    implicit_nonces: MaybeInactive<Nonces>,
    public_keys: IterableMap<PublicKey, MaybeInactive<Nonces>>,

    pub state: AccountState,

    prefix: Vec<u8>,
}

impl Account {
    #[inline]
    pub fn new<S>(prefix: S) -> Self
    where
        S: IntoStorageKey,
    {
        let prefix = prefix.into_storage_key();

        Self {
            implicit_nonces: Nonces::new(prefix.as_slice().nest(AccountPrefix::ImplicitNonces))
                .into(),
            public_keys: IterableMap::new(prefix.as_slice().nest(AccountPrefix::PublicKeys)),
            state: AccountState::new(prefix.as_slice().nest(AccountPrefix::State)),
            prefix,
        }
    }

    #[inline]
    pub fn add_public_key(&mut self, me: &AccountId, public_key: PublicKey) -> &mut Nonces {
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
        .activate()
    }

    #[inline]
    pub fn deactivate_public_key(&mut self, me: &AccountId, public_key: &PublicKey) {
        if me == &public_key.to_implicit_account_id() {
            Some(&mut self.implicit_nonces)
        } else {
            self.public_keys.get_mut(public_key)
        }
        .map(MaybeInactive::deactivate);
    }

    #[inline]
    pub fn iter_public_keys(&self) -> impl Iterator<Item = &'_ PublicKey> {
        self.public_keys.keys()
    }

    #[must_use]
    #[inline]
    pub fn public_key_nonces(&self, me: &AccountId, public_key: &PublicKey) -> Option<&Nonces> {
        if me == &public_key.to_implicit_account_id() {
            Some(&self.implicit_nonces)
        } else {
            self.public_keys.get(public_key)
        }
        .and_then(MaybeInactive::as_active)
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
        .and_then(MaybeInactive::as_active_mut)
    }

    pub fn verify_signed_as_nep413<S, T>(
        &mut self,
        me: &AccountId,
        signed: SignedPayload<S>,
    ) -> Result<T, DefuseError>
    where
        S: Payload + ValidatePayloadAs<Nep413Payload<T>, Error: Into<DefuseError>>,
    {
        let public_key = signed.verify().ok_or(DefuseError::InvalidSignature)?;

        let payload: Nep413Payload<T> = signed.payload.validate_as().map_err(Into::into)?;
        if payload.recipient != *CURRENT_ACCOUNT_ID {
            return Err(DefuseError::WrongRecipient);
        }

        self.public_key_nonces_mut(me, &public_key)
            .ok_or(DefuseError::InvalidSignature)?
            .commit(payload.nonce)?;

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

#[derive(Debug, Clone, Copy, Default)]
#[near(serializers = [borsh])]
struct MaybeInactive<T> {
    inactive: bool,
    inner: T,
}

impl<T> MaybeInactive<T> {
    #[inline]
    pub const fn active(inner: T) -> Self {
        Self {
            inactive: false,
            inner,
        }
    }

    #[inline]
    pub fn activate(&mut self) -> &mut T {
        self.inactive = false;
        &mut self.inner
    }

    #[inline]
    pub fn deactivate(&mut self) {
        self.inactive = true;
    }

    #[inline]
    pub const fn is_active(&self) -> bool {
        !self.inactive
    }

    #[inline]
    pub const fn as_active(&self) -> Option<&T> {
        if self.is_active() {
            Some(&self.inner)
        } else {
            None
        }
    }

    #[inline]
    pub fn as_active_mut(&mut self) -> Option<&mut T> {
        if self.is_active() {
            Some(&mut self.inner)
        } else {
            None
        }
    }
}

impl<T> From<T> for MaybeInactive<T> {
    fn from(value: T) -> Self {
        Self::active(value)
    }
}
