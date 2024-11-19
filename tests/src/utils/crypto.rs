use defuse::core::nep413::{Nep413Payload, SignedNep413Payload};
use near_workspaces::Account;

pub trait Signer {
    fn sign_nep413(&self, payload: Nep413Payload) -> SignedNep413Payload;
}

impl Signer for Account {
    fn sign_nep413(&self, payload: Nep413Payload) -> SignedNep413Payload {
        // near_sdk does not expose near_crypto API
        let secret_key: near_crypto::SecretKey = self.secret_key().to_string().parse().unwrap();

        match (secret_key.sign(data), secret_key.public_key()) {
            (near_crypto::Signature::ED25519(sig), near_crypto::PublicKey::ED25519(pk)) => {
                Signature::Ed25519 {
                    signature: sig.to_bytes(),
                    public_key: pk.0,
                }
            }
            (near_crypto::Signature::SECP256K1(sig), near_crypto::PublicKey::SECP256K1(_pk)) => {
                Signature::Secp256k1 {
                    signature: sig.into(),
                }
            }
            _ => unreachable!(),
        }
    }
}
