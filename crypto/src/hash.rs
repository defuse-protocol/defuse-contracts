use near_sdk::env;

pub trait Hasher {
    type Output: AsRef<[u8]>;

    fn hash(&self, data: &[u8]) -> Self::Output;
}

pub struct Sha256;

impl Hasher for Sha256 {
    type Output = [u8; 32];

    #[inline]
    fn hash(&self, data: &[u8]) -> Self::Output {
        env::sha256_array(data)
    }
}

pub struct Keccak256;

impl Hasher for Keccak256 {
    type Output = [u8; 32];

    #[inline]
    fn hash(&self, data: &[u8]) -> Self::Output {
        env::keccak256_array(data)
    }
}

pub struct Keccak512;

impl Hasher for Keccak512 {
    type Output = [u8; 64];

    #[inline]
    fn hash(&self, data: &[u8]) -> Self::Output {
        env::keccak512_array(data)
    }
}
