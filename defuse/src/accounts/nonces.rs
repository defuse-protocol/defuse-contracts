use defuse_contracts::{
    defuse::DefuseError,
    utils::bitmap::{BitMap256, Uint256},
};
use near_sdk::{near, IntoStorageKey};

pub type Nonce = Uint256;

#[derive(Debug)]
#[near(serializers = [borsh])]
pub struct Nonces(BitMap256);

impl Nonces {
    #[inline]
    pub fn new<S>(prefix: S) -> Self
    where
        S: IntoStorageKey,
    {
        Self(BitMap256::new(prefix))
    }

    #[must_use]
    #[inline]
    pub fn is_used(&self, n: Nonce) -> bool {
        self.0.get(n)
    }

    #[inline]
    pub fn commit(&mut self, n: Nonce) -> Result<(), DefuseError> {
        if self.0.set(n) {
            return Err(DefuseError::NonceUsed);
        }
        Ok(())
    }
}
