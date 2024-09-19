use impl_tools::autoimpl;
use near_sdk::near;

use super::{PublicKey, Signature};

pub trait Payload {
    fn hash(&self) -> [u8; 32];
}

#[derive(Debug, Clone)]
#[near(serializers = [borsh, json])]
#[autoimpl(Deref using self.payload)]
pub struct Signed<T> {
    #[serde(flatten)]
    pub payload: T,
    #[serde(flatten)]
    pub signature: Signature,
}

impl<T> Signed<T>
where
    T: Payload,
{
    /// Veirifies the signature and returns public counterpart of the key
    /// that was used to sign the payload or `None` if the signature is
    /// invalid
    #[must_use]
    #[inline]
    pub fn verify(&self) -> Option<PublicKey> {
        self.signature.verify(&self.payload.hash())
    }
}
