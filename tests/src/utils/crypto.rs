use defuse_contracts::{
    crypto::{Curve, Ed25519, Payload},
    nep413::{Nep413Payload, SignedNep413Payload},
};
use near_workspaces::Account;

pub trait Signer {
    fn public_key(&self) -> <Ed25519 as Curve>::PublicKey;
    fn sign_ed25519(&self, data: &[u8]) -> <Ed25519 as Curve>::Signature;

    fn sign_nep413(&self, payload: Nep413Payload) -> SignedNep413Payload;
}

impl Signer for Account {
    fn public_key(&self) -> <Ed25519 as Curve>::PublicKey {
        Ed25519::parse_base58(self.secret_key().public_key().to_string()).unwrap()
    }

    fn sign_ed25519(&self, data: &[u8]) -> <Ed25519 as Curve>::Signature {
        // near_sdk does not expose near_crypto API
        let secret_key: near_crypto::SecretKey = self.secret_key().to_string().parse().unwrap();

        let near_crypto::Signature::ED25519(sig) = secret_key.sign(data) else {
            panic!("only ed25519 signatures are supported");
        };
        sig.to_bytes()
    }

    fn sign_nep413(&self, payload: Nep413Payload) -> SignedNep413Payload {
        SignedNep413Payload {
            public_key: self.public_key(),
            signature: self.sign_ed25519(&payload.hash()),
            payload,
        }
    }
}
