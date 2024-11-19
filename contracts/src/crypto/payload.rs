use std::ops::Deref;

use near_sdk::CryptoHash;

use super::Curve;

pub trait Payload {
    fn hash(&self) -> CryptoHash;
}

pub trait SignedPayload: Deref<Target = Self::Payload> {
    type Payload: Payload;
    type Curve: Curve;

    fn verify(&self) -> Option<<Self::Curve as Curve>::PublicKey>;
}
