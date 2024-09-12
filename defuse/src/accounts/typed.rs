use core::ops::{Deref, DerefMut};

use defuse_contracts::{
    crypto::{Payload, PublicKey, Signed},
    defuse::{verify::payload::ValidatePayloadAs, DefuseError},
    nep413::Nep413Payload,
    utils::cache::CURRENT_ACCOUNT_ID,
};
use near_sdk::{near, AccountId};

use super::{Account, NamedAccount};

#[derive(Debug)]
#[near(serializers = [borsh])]
pub enum TypedAccount {
    Implicit(Account),
    Named(NamedAccount),
}

impl TypedAccount {
    #[inline]
    pub const fn as_named(&self) -> Option<&NamedAccount> {
        match self {
            Self::Named(named) => Some(named),
            _ => None,
        }
    }

    #[inline]
    pub fn as_named_mut(&mut self) -> Option<&mut NamedAccount> {
        match self {
            Self::Named(named) => Some(named),
            _ => None,
        }
    }

    pub fn has_public_key(&self, account_id: &AccountId, public_key: &PublicKey) -> bool {
        match self {
            TypedAccount::Implicit(_) => account_id == &public_key.to_implicit_account_id(),
            TypedAccount::Named(account) => account.has_public_key(public_key),
        }
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
        if !self.has_public_key(account_id, &public_key) {
            return Err(DefuseError::InvalidSignature);
        }

        let payload = signed.payload.validate_as().map_err(Into::into)?;

        if payload.recipient != *CURRENT_ACCOUNT_ID {
            return Err(DefuseError::WrongRecipient);
        }

        self.commit_nonce(payload.nonce)?;

        Ok(payload.message)
    }
}

impl Deref for TypedAccount {
    type Target = Account;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Implicit(implicit) => implicit,
            Self::Named(named) => named.deref(),
        }
    }
}

impl DerefMut for TypedAccount {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Implicit(implicit) => implicit,
            Self::Named(named) => named.deref_mut(),
        }
    }
}
