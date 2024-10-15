use near_sdk::{near, store::LookupMap, IntoStorageKey};

pub type U256 = [u8; 32];
type U248 = [u8; 31];

/// 256-bit map.  
/// See [permit2 nonce schema](https://docs.uniswap.org/contracts/permit2/reference/signature-transfer#nonce-schema)
#[derive(Debug)]
#[near(serializers = [borsh])]
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
        let [word_pos @ .., bit_pos] = n;
        let Some(bitmap) = self.0.get(&word_pos) else {
            return false;
        };
        let byte = bitmap[(bit_pos / 8) as usize];
        let byte_mask = 1 << (bit_pos % 8);
        byte & byte_mask != 0
    }

    #[inline]
    fn get_mut_byte_with_mask(&mut self, n: U256) -> (&mut u8, u8) {
        let [word_pos @ .., bit_pos] = n;
        let bitmap = self.0.entry(word_pos).or_default();
        let byte = &mut bitmap[(bit_pos / 8) as usize];
        let byte_mask = 1 << (bit_pos % 8);
        (byte, byte_mask)
    }

    #[inline]
    pub fn set_to(&mut self, n: U256, v: bool) -> bool {
        if v {
            self.set(n)
        } else {
            self.clear(n)
        }
    }

    /// Set the bit `n` and return old value
    #[inline]
    pub fn set(&mut self, n: U256) -> bool {
        let (byte, mask) = self.get_mut_byte_with_mask(n);
        let old = *byte & mask != 0;
        *byte |= mask;
        old
    }

    /// Clear the bit `n` and return old value
    #[inline]
    pub fn clear(&mut self, n: U256) -> bool {
        let (byte, mask) = self.get_mut_byte_with_mask(n);
        let old = *byte & mask != 0;
        *byte &= !mask;
        old
    }

    /// Toggle the bit `n` and return old value
    #[inline]
    pub fn toggle(&mut self, n: U256) -> bool {
        let (byte, mask) = self.get_mut_byte_with_mask(n);
        let old = *byte & mask != 0;
        *byte ^= mask;
        old
    }
}

#[cfg(test)]
mod tests {
    use bnum::BUintD8;

    use super::*;

    #[test]
    fn test() {
        let mut m = BitMap256::new(0);

        type U256 = BUintD8<32>;

        for n in [U256::ZERO, U256::ONE, U256::MAX - U256::ONE, U256::MAX].map(Into::into) {
            assert!(!m.get(n));

            assert!(!m.set(n));
            assert!(m.get(n));
            assert!(m.set(n));
            assert!(m.get(n));

            assert!(m.clear(n));
            assert!(!m.get(n));
            assert!(!m.clear(n));
            assert!(!m.get(n));
        }
    }
}
