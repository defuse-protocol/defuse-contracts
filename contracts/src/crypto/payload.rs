pub trait Payload {
    fn hash(&self) -> [u8; 32];
}

pub trait SignedPayload {
    type PublicKey;

    fn verify(&self) -> Option<Self::PublicKey>;
}
