use core::iter;

use defuse_contracts::{
    defuse::{DefuseError, Result},
    nep413::Nonce,
    utils::bitmap::BitMap256,
};
use near_sdk::{near, IntoStorageKey};

/// See [permit2 nonce schema](https://docs.uniswap.org/contracts/permit2/reference/signature-transfer#nonce-schema)
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
    fn iter_unused(&self, start: Nonce) -> impl Iterator<Item = Nonce> + '_ {
        iter::successors(Some(start), |n| n.checked_add(Nonce::ONE)).filter(|n| !self.is_used(*n))
    }

    /// Returns the first nonce available starting from `start` or `0` otherwise.
    #[inline]
    pub fn next_unused(&self, start: impl Into<Option<Nonce>>) -> Option<Nonce> {
        self.iter_unused(start.into().unwrap_or_default()).next()
    }

    #[inline]
    pub fn commit(&mut self, n: Nonce) -> Result<()> {
        if self.0.set(n, true) {
            return Err(DefuseError::NonceUsed);
        }
        Ok(())
    }
}
