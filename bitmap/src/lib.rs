use defuse_map_utils::Map;
use near_sdk::near;

pub type U256 = [u8; 32];
pub type U248 = [u8; 31];

/// 256-bit map.  
/// See [permit2 nonce schema](https://docs.uniswap.org/contracts/permit2/reference/signature-transfer#nonce-schema)
#[near(serializers = [borsh, json])]
#[derive(Debug, Clone, Default)]
pub struct BitMap256<T: Map<K = U248, V = U256>>(T);

impl<T> BitMap256<T>
where
    T: Map<K = U248, V = U256>,
{
    #[inline]
    pub const fn new(map: T) -> Self {
        Self(map)
    }

    /// Get the bit `n`
    #[inline]
    pub fn get_bit(&self, n: U256) -> bool {
        let [word_pos @ .., bit_pos] = n;
        let Some(bitmap) = self.0.get(&word_pos) else {
            return false;
        };
        let byte = bitmap[usize::from(bit_pos / 8)];
        let byte_mask = 1 << (bit_pos % 8);
        byte & byte_mask != 0
    }

    #[inline]
    fn get_mut_byte_with_mask(&mut self, n: U256) -> (&mut u8, u8) {
        let [word_pos @ .., bit_pos] = n;
        let bitmap = self.0.entry(word_pos).or_default();
        let byte = &mut bitmap[usize::from(bit_pos / 8)];
        let byte_mask = 1 << (bit_pos % 8);
        (byte, byte_mask)
    }

    /// Set the bit `n` and return old value
    #[inline]
    pub fn set_bit(&mut self, n: U256) -> bool {
        let (byte, mask) = self.get_mut_byte_with_mask(n);
        let old = *byte & mask != 0;
        *byte |= mask;
        old
    }

    /// Clear the bit `n` and return old value
    #[inline]
    pub fn clear_bit(&mut self, n: U256) -> bool {
        let (byte, mask) = self.get_mut_byte_with_mask(n);
        let old = *byte & mask != 0;
        *byte &= !mask;
        old
    }

    /// Toggle the bit `n` and return old value
    #[inline]
    pub fn toggle_bit(&mut self, n: U256) -> bool {
        let (byte, mask) = self.get_mut_byte_with_mask(n);
        let old = *byte & mask != 0;
        *byte ^= mask;
        old
    }

    /// Set bit `n` to given value
    #[inline]
    pub fn set_bit_to(&mut self, n: U256, v: bool) -> bool {
        if v {
            self.set_bit(n)
        } else {
            self.clear_bit(n)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use bnum::BUintD8;

    use super::*;

    #[test]
    fn test() {
        let mut m = BitMap256::<HashMap<U248, U256>>::default();

        type N = BUintD8<32>;

        for n in [N::ZERO, N::ONE, N::MAX - N::ONE, N::MAX].map(Into::into) {
            assert!(!m.get_bit(n));

            assert!(!m.set_bit(n));
            assert!(m.get_bit(n));
            assert!(m.set_bit(n));
            assert!(m.get_bit(n));

            assert!(m.clear_bit(n));
            assert!(!m.get_bit(n));
            assert!(!m.clear_bit(n));
            assert!(!m.get_bit(n));
        }
    }
}
