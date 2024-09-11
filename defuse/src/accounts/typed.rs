use core::ops::{Deref, DerefMut};

use defuse_contracts::{
    defuse::DefuseError,
    nep413::{PublicKey, SignedPayload},
    utils::cache::CURRENT_ACCOUNT_ID,
};
use near_sdk::{borsh::BorshSerialize, near, AccountId};

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

    pub fn verify_nep413<T>(
        &mut self,
        account_id: &AccountId,
        signed: SignedPayload<T>,
    ) -> Result<T, DefuseError>
    where
        T: BorshSerialize,
    {
        if signed.recipient != *CURRENT_ACCOUNT_ID {
            return Err(DefuseError::WrongRecipient);
        }

        let public_key = signed.verify().ok_or(DefuseError::InvalidSignature)?;
        if !self.has_public_key(account_id, &public_key) {
            return Err(DefuseError::InvalidSignature);
        }

        self.commit_nonce(signed.payload.nonce)?;

        Ok(signed.payload.message)
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
