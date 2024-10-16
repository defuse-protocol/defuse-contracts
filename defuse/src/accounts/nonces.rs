use defuse_contracts::{
    defuse::{DefuseError, Result},
    nep413::U256,
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
    pub fn is_used(&self, n: U256) -> bool {
        self.0.get(n)
    }

    #[inline]
    pub fn commit(&mut self, n: U256) -> Result<()> {
        if self.0.set(n) {
            return Err(DefuseError::NonceUsed);
        }
        Ok(())
    }
}
