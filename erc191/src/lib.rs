use std::borrow::Cow;

use defuse_crypto::{
    serde::AsCurve, CryptoHash, Curve, Keccak256, Payload, PublicKey, Secp256k1, SignedPayload,
};
use impl_tools::autoimpl;
use near_sdk::{env, near};
use serde_with::serde_as;

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct Erc191Payload(pub String);

impl Erc191Payload {
    #[inline]
    pub fn prehash(&self) -> Vec<u8> {
        let data = self.0.as_bytes();
        [
            format!("\x19Ethereum Signed Message:\n{}", data.len()).as_bytes(),
            data,
        ]
        .concat()
    }
}

impl Payload for Erc191Payload {
    #[inline]
    fn hash(&self) -> CryptoHash {
        env::keccak256_array(&self.prehash())
    }
}

// impl Payload for Erc191Payload {
//     type Curve = Secp256k1;
//     type Hasher = Keccak256;

//     #[inline]
//     fn curve(&self) -> Self::Curve {
//         Secp256k1
//     }

//     #[inline]
//     fn serialize(&self) -> Cow<'_, [u8]> {
//         let data = self.0.as_bytes();
//         [
//             format!("\x19Ethereum Signed Message:\n{}", data.len()).as_bytes(),
//             data,
//         ]
//         .concat()
//         .into()
//     }

//     #[inline]
//     fn pre_sign(&self) -> impl AsRef<<Self::Curve as Curve>::Message> {
//         Cow::Owned(self.hash())
//     }

//     #[inline]
//     fn hasher(&self) -> Self::Hasher {
//         Keccak256
//     }
// }

#[cfg_attr(
    all(feature = "abi", not(target_arch = "wasm32")),
    serde_as(schemars = true)
)]
#[cfg_attr(
    not(all(feature = "abi", not(target_arch = "wasm32"))),
    serde_as(schemars = false)
)]
#[near(serializers = [borsh, json])]
#[autoimpl(Deref using self.payload)]
#[derive(Debug, Clone)]
pub struct SignedErc191Payload {
    pub payload: Erc191Payload,

    #[serde_as(as = "AsCurve<Secp256k1>")]
    pub signature: <Secp256k1 as Curve>::Signature,
}

impl Payload for SignedErc191Payload {
    #[inline]
    fn hash(&self) -> CryptoHash {
        self.payload.hash()
    }
}

impl SignedPayload for SignedErc191Payload {
    type PublicKey = <Secp256k1 as Curve>::PublicKey;

    fn verify(&self) -> Option<Self::PublicKey> {
        let [signature @ .., v] = &self.signature;
        env::ecrecover(
            &self.payload.hash(),
            signature,
            *v,
            // Do not accept malleabile signatures:
            // https://github.com/near/nearcore/blob/d73041cc1d1a70af4456fceefaceb1bf7f684fde/core/crypto/src/signature.rs#L448-L455
            true,
        )
    }
}

// impl SignedPayload for SignedErc191Payload {
//     type Payload = Erc191Payload;

//     #[inline]
//     fn payload(&self) -> &Self::Payload {
//         &self.payload
//     }

//     #[inline]
//     fn verifying_key(&self) -> &<<Self::Payload as Payload>::Curve as Curve>::VerifyingKey {
//         &()
//     }

//     #[inline]
//     fn signature(&self) -> &<<Self::Payload as Payload>::Curve as Curve>::Signature {
//         &self.signature
//     }
// }
