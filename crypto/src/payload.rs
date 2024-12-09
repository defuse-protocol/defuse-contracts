pub use near_sdk::CryptoHash;

pub trait Payload {
    fn hash(&self) -> CryptoHash;
}

pub trait SignedPayload: Payload {
    type PublicKey;

    fn verify(&self) -> Option<Self::PublicKey>;
}
