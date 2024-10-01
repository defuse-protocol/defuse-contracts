use bnum::BUintD8;
use near_sdk::{near, store::LookupMap, IntoStorageKey};

use super::integer::U256;

type U248 = BUintD8<31>;

/// 256-bit map.  
/// See [permit2 nonce schema](https://docs.uniswap.org/contracts/permit2/reference/signature-transfer#nonce-schema)
#[derive(Debug)]
#[near(serializers = [borsh])]
// TODO: hasher Identity?
pub struct BitMap256(LookupMap<U248, U256>);

impl BitMap256 {
    #[inline]
    pub fn new<S>(prefix: S) -> Self
    where
        S: IntoStorageKey,
    {
        Self(LookupMap::new(prefix))
    }

    /// Get the bit `n`
    #[inline]
    pub fn get(&self, n: U256) -> bool {
        let (word_pos, bit_pos) = Self::split_word_bit_pos(n);
        let Some(bitmap) = self.0.get(&word_pos) else {
            return false;
        };
        bitmap.bit(bit_pos as u32)
    }

    /// Set the bit `n` to given `value` and return old value
    #[inline]
    pub fn set(&mut self, n: U256, value: bool) -> bool {
        let (word_pos, bit_pos) = Self::split_word_bit_pos(n);
        let b = self.0.entry(word_pos).or_default();
        let old = b.bit(bit_pos as u32);
        b.set_bit(bit_pos as u32, value);
        old
    }

    fn split_word_bit_pos(n: U256) -> (U248, u8) {
        // U256 is stored as little-endian,
        // so `bit_pos` is stored in the least-significant byte
        let [bit_pos, word_pos @ ..]: [u8; 32] = n.into();
        (word_pos.into(), bit_pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut m = BitMap256::new(0);

        for n in [U256::ZERO, U256::ONE, U256::MAX - U256::ONE, U256::MAX] {
            assert!(!m.get(n));

            assert!(!m.set(n, true));
            assert!(m.get(n));
            assert!(m.set(n, true));
            assert!(m.get(n));

            assert!(m.set(n, false));
            assert!(!m.get(n));
            assert!(!m.set(n, false));
            assert!(!m.get(n));
        }
    }
}
