use defuse_contracts::{
    crypto::{Payload, PublicKey, Signed},
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
    prefix: Vec<u8>,
    public_keys: IterableMap<PublicKey, Nonces>,

    pub state: AccountState,
}

impl Account {
    #[inline]
    pub fn new<S>(prefix: S) -> Self
    where
        S: IntoStorageKey,
    {
        let prefix = prefix.into_storage_key();

        Self {
            public_keys: IterableMap::new(prefix.as_slice().nest(AccountPrefix::PublicKeys)),
            state: AccountState::new(prefix.as_slice().nest(AccountPrefix::State)),
            prefix,
        }
    }

    #[inline]
    pub fn iter_public_keys(&self) -> impl Iterator<Item = &'_ PublicKey> {
        self.public_keys.keys()
    }

    #[must_use]
    #[inline]
    pub fn public_key_nonces(&self, public_key: &PublicKey) -> Option<&Nonces> {
        self.public_keys.get(public_key)
    }

    #[must_use]
    #[inline]
    pub fn public_key_nonces_mut(&mut self, public_key: &PublicKey) -> Option<&mut Nonces> {
        self.public_keys.get_mut(public_key)
    }

    #[inline]
    pub fn add_public_key(&mut self, public_key: PublicKey) -> &mut Nonces {
        self.public_keys
            .entry(public_key)
            .or_insert_with_key(|public_key| {
                Nonces::new(
                    self.prefix
                        .as_slice()
                        .nest(AccountPrefix::PublicKeyNonces(public_key)),
                )
            })
    }

    #[inline]
    pub fn remove_public_key(&mut self, public_key: &PublicKey) -> bool {
        self.public_keys.remove(public_key).is_some()
    }
}

#[autoimpl(Deref using self.account)]
#[autoimpl(DerefMut using self.account)]
pub struct MaybeFreshAccount<'a> {
    fresh: bool,
    account: &'a mut Account,
}

impl<'a> MaybeFreshAccount<'a> {
    #[inline]
    pub(super) fn new(account: &'a mut Account, fresh: bool) -> Self {
        Self { fresh, account }
    }

    #[inline]
    pub fn into_account(self) -> &'a mut Account {
        self.account
    }

    pub fn verify_signed_as_nep413<S, T>(
        &mut self,
        account_id: &AccountId,
        signed: Signed<S>,
    ) -> Result<T, DefuseError>
    where
        S: Payload + ValidatePayloadAs<Nep413Payload<T>, Error: Into<DefuseError>>,
    {
        let public_key = signed.verify().ok_or(DefuseError::InvalidSignature)?;

        let payload: Nep413Payload<T> = signed.payload.validate_as().map_err(Into::into)?;
        if payload.recipient != *CURRENT_ACCOUNT_ID {
            return Err(DefuseError::WrongRecipient);
        }

        if self.fresh {
            // TODO: what if this implicit account has changed his FullAccessKey already?
            if account_id != &public_key.to_implicit_account_id() {
                return Err(DefuseError::InvalidSignature);
            }
            self.account.add_public_key(public_key)
        } else {
            self.public_key_nonces_mut(&public_key)
                .ok_or(DefuseError::InvalidSignature)?
        }
        .commit(payload.nonce)?;
        self.fresh = false;

        Ok(payload.message)
    }
}

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "::near_sdk::borsh")]
enum AccountPrefix<'a> {
    PublicKeys,
    PublicKeyNonces(&'a PublicKey),
    State,
}
