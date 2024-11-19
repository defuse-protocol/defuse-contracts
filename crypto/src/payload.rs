// use std::borrow::Cow;

// use crate::{hash::Hasher, Curve};

pub use near_sdk::CryptoHash;

pub trait Payload {
    fn hash(&self) -> CryptoHash;
}

pub trait SignedPayload: Payload {
    type PublicKey;

    fn verify(&self) -> Option<Self::PublicKey>;
}

// pub trait Payload {
//     /// Curve used by this signing standard
//     type Curve: Curve;
//     /// Hasher used by this signing standard
//     type Hasher: Hasher;

//     fn curve(&self) -> Self::Curve;
//     fn serialize(&self) -> Cow<'_, [u8]>;

//     /// Return message to be signed/verified via [`Self::Curve`](Payload::Curve)
//     fn pre_sign(&self) -> impl AsRef<<Self::Curve as Curve>::Message>;
//     /// Veirfy a signature on this payload
//     #[inline]
//     fn verify(
//         &self,
//         signature: &<Self::Curve as Curve>::Signature,
//         verifying_key: &<Self::Curve as Curve>::VerifyingKey,
//     ) -> Option<<Self::Curve as Curve>::PublicKey> {
//         self.curve()
//             .verify(signature, self.pre_sign().as_ref(), verifying_key)
//     }

//     fn hasher(&self) -> Self::Hasher;

//     /// Hash this payload using [`Self::Hasher`](Payload::Hasher)
//     #[inline]
//     fn hash(&self) -> <Self::Hasher as Hasher>::Output {
//         self.hasher().hash(self.serialize().as_ref().as_ref())
//     }
// }

// pub trait SignedPayload {
//     type Payload: Payload;

//     fn payload(&self) -> &Self::Payload;

//     fn verifying_key(&self) -> &<<Self::Payload as Payload>::Curve as Curve>::VerifyingKey;
//     fn signature(&self) -> &<<Self::Payload as Payload>::Curve as Curve>::Signature;

//     #[inline]
//     fn verify(&self) -> Option<<<Self::Payload as Payload>::Curve as Curve>::PublicKey> {
//         self.payload()
//             .verify(self.signature(), self.verifying_key())
//     }
// }
