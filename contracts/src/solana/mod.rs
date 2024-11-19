use impl_tools::autoimpl;
use near_sdk::{env, near};
use serde_with::serde_as;

use crate::{
    crypto::{AsCurve, Curve, Ed25519, Payload, SignedPayload},
    utils::serde::base58::Base58,
};

/// See https://github.com/solana-labs/solana/blob/27eff8408b7223bb3c4ab70523f8a8dca3ca6645/sdk/src/offchain_message.rs#L167
#[near(serializers = [borsh, json])]
#[serde(tag = "version", rename_all = "snake_case")]
#[derive(Debug, Clone)]
pub enum OffchainMessage {
    // TODO: rename?
    V0(OffchainMessageV0),
}

impl OffchainMessage {
    pub const SIGNING_DOMAIN: &[u8] = b"\xffsolana offchain";

    pub fn signer(&self) -> Option<&<Ed25519 as Curve>::PublicKey> {
        match self {
            Self::V0(message) => message.signers.first(),
        }
    }
}

impl Payload for OffchainMessage {
    fn hash(&self) -> [u8; 32] {
        todo!()
    }
}

#[serde_as]
// TODO: borsh
#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct OffchainMessageV0 {
    #[serde_as(as = "Base58")]
    pub application_domain: [u8; 32],
    pub message_format: u8,
    #[serde_as(as = "Vec<AsCurve<Ed25519>>")]
    pub signers: Vec<<Ed25519 as Curve>::PublicKey>,
    pub message: Vec<u8>,
}

impl Payload for OffchainMessageV0 {
    fn hash(&self) -> near_sdk::CryptoHash {
        todo!()
    }
}

/// See https://github.com/solana-labs/solana/blob/master/docs/src/proposals/off-chain-message-signing.md#message-format
#[near(serializers = [json])]
#[serde()] // TODO
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum OffchainMessageFormat {
    RestrictedAscii = 0,
    LimitedUtf8 = 1,
    ExtendedUtf8 = 2,
}

#[serde_as]
#[near(serializers = [borsh, json])]
#[autoimpl(Deref using self.message)]
#[derive(Debug, Clone)]
pub struct SignedOffchainMessage {
    pub message: OffchainMessage,
    #[serde_as(as = "Vec<AsCurve<Ed25519>>")]
    pub signatures: Vec<<Ed25519 as Curve>::Signature>,
}

impl Payload for SignedOffchainMessage {
    fn hash(&self) -> near_sdk::CryptoHash {
        todo!()
    }
}

impl SignedPayload for SignedOffchainMessage {
    type Curve = Ed25519;

    fn verify(&self) -> Option<<Self::Curve as Curve>::PublicKey> {
        let data = self.hash(); // TODO
        match &self.message {
            OffchainMessage::V0(message) => {
                let signer = message.signers.first()?;
                if self.signatures.len() != message.signers.len() {
                    return None;
                }
                self.signatures
                    .iter()
                    .zip(&message.signers)
                    // verify all signers and their signatures
                    .all(|(sign, pk)| env::ed25519_verify(sign, &data, pk))
                    // but return only the first signer
                    .then_some(signer)
                    .cloned()
            }
        }
    }
}
