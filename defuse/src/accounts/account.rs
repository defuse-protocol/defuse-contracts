use defuse_contracts::{
    defuse::DefuseError,
    utils::{
        bitmap::{BitMap256, Uint256},
        prefix::NestPrefix,
    },
};
use impl_tools::autoimpl;
use near_sdk::{near, BorshStorageKey, IntoStorageKey};

use super::AccountState;

pub type Nonce = Uint256;

#[derive(Debug)]
#[near(serializers = [borsh])]
#[autoimpl(Deref using self.state)]
#[autoimpl(DerefMut using self.state)]
pub struct Account {
    nonces: BitMap256,
    pub state: AccountState,
}

impl Account {
    #[inline]
    pub fn new<S>(prefix: S) -> Self
    where
        S: IntoStorageKey,
    {
        let parent = prefix.into_storage_key();

        #[derive(BorshStorageKey)]
        #[near(serializers = [borsh])]
        enum AccountPrefix {
            Nonces,
            State,
        }

        Self {
            nonces: BitMap256::new(parent.as_slice().nest(AccountPrefix::Nonces)),
            state: AccountState::new(parent.as_slice().nest(AccountPrefix::State)),
        }
    }

    #[inline]
    pub fn commit_nonce(&mut self, n: Nonce) -> Result<(), DefuseError> {
        if self.nonces.set(n) {
            return Err(DefuseError::NonceUsed);
        }
        Ok(())
    }
}
