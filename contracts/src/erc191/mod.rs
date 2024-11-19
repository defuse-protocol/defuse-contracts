use near_sdk::{env, near, CryptoHash};
use serde_with::serde_as;

use crate::crypto::{AsCurve, Curve, Payload, Secp256k1, SignedPayload};

#[cfg_attr(
    all(feature = "abi", not(target_arch = "wasm32")),
    serde_as(schemars = true)
)]
#[cfg_attr(
    not(all(feature = "abi", not(target_arch = "wasm32"))),
    serde_as(schemars = false)
)]
#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct SignedErc191Payload {
    pub payload: String,

    #[serde_as(as = "AsCurve<Secp256k1>")]
    pub signature: <Secp256k1 as Curve>::Signature,
}

impl Payload for SignedErc191Payload {
    #[inline]
    fn hash(&self) -> CryptoHash {
        sign_hash(self.payload.as_bytes())
    }
}

impl SignedPayload for SignedErc191Payload {
    type PublicKey = <Secp256k1 as Curve>::PublicKey;

    fn verify(&self) -> Option<Self::PublicKey> {
        let [signature @ .., v] = &self.signature;
        env::ecrecover(
            &self.hash(),
            signature,
            *v,
            // Do not accept malleabile signatures:
            // https://github.com/near/nearcore/blob/d73041cc1d1a70af4456fceefaceb1bf7f684fde/core/crypto/src/signature.rs#L448-L455
            true,
        )
    }
}

/// See [personal_sign](https://github.com/ethereum/ercs/blob/master/ERCS/erc-191.md#version-0x45-e)
#[inline]
pub fn sign_hash(data: &[u8]) -> [u8; 32] {
    env::keccak256_array(
        // 0x19 <0x45 (E)> <thereum Signed Message:\n" + len(message)> <data to sign>
        &[
            format!("\x19Ethereum Signed Message:\n{}", data.len()).as_bytes(),
            data,
        ]
        .concat(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use hex_literal::hex;

    #[test]
    fn test_hash() {
        assert_eq!(
            sign_hash(b"Please sign this message to confirm your identity."),
            hex!("c21712258067502aad461ea687c066dfebd518e90f5b57d4cc04f5b3eb34f00e")
        );
    }
}
