use core::iter;
use std::io;

use defuse_crypto::{serde::AsCurve, Curve, Ed25519, Payload, SignedPayload};
use defuse_serde_utils::base58::Base58;
use impl_tools::autoimpl;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env, near, CryptoHash,
};
use serde_with::serde_as;

#[near(serializers = [json])]
#[serde(tag = "version")]
#[derive(Debug, Clone)]
pub enum SolanaOffchainMessage {
    V0(SolanaOffchainMessageV0),
}

impl SolanaOffchainMessage {
    pub const SIGNING_DOMAIN: &[u8] = b"\xffsolana offchain";

    pub fn signers(&self) -> &[<Ed25519 as Curve>::PublicKey] {
        match self {
            Self::V0(message) => &message.signers,
        }
    }
}

impl Payload for SolanaOffchainMessage {
    #[inline]
    fn hash(&self) -> CryptoHash {
        env::sha256_array(&borsh::to_vec(self).unwrap_or_else(|_| unreachable!()))
    }
}

impl BorshSerialize for SolanaOffchainMessage {
    fn serialize<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(Self::SIGNING_DOMAIN)?;
        match self {
            Self::V0(message) => (0u8, message).serialize(writer),
        }
    }
}

impl BorshDeserialize for SolanaOffchainMessage {
    fn deserialize_reader<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        let mut buf = [0u8; Self::SIGNING_DOMAIN.len()];
        reader.read_exact(&mut buf)?;
        if buf != Self::SIGNING_DOMAIN {
            return Err(io::Error::other("wrong singing domain"));
        }

        match u8::deserialize_reader(reader)? {
            0 => BorshDeserialize::deserialize_reader(reader).map(Self::V0),
            _ => Err(io::Error::other("unknown header version")),
        }
    }
}

#[cfg_attr(
    all(feature = "abi", not(target_arch = "wasm32")),
    serde_as(schemars = true)
)]
#[cfg_attr(
    not(all(feature = "abi", not(target_arch = "wasm32"))),
    serde_as(schemars = false)
)]
#[near(serializers = [json])]
#[derive(Debug, Clone)]
pub struct SolanaOffchainMessageV0 {
    #[serde_as(as = "Base58")]
    pub application_domain: [u8; 32],
    pub message_format: MessageFormat,
    pub message: String,
    #[serde_as(as = "Vec<AsCurve<Ed25519>>")]
    pub signers: Vec<<Ed25519 as Curve>::PublicKey>,
}

impl BorshSerialize for SolanaOffchainMessageV0 {
    fn serialize<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.application_domain)?;

        self.message_format.serialize(writer)?;

        let signer_count: u8 = self
            .signers
            .len()
            .try_into()
            .map_err(|_| io::Error::other("too many signers"))?;
        signer_count.serialize(writer)?;
        for signer in &self.signers {
            signer.serialize(writer)?;
        }

        let message_length: u16 = self
            .message
            .len()
            .try_into()
            .map_err(|_| io::Error::other("message is too long"))?;
        // TODO: verify format
        message_length.serialize(writer)?;
        writer.write_all(self.message.as_bytes())?;

        Ok(())
    }
}

impl BorshDeserialize for SolanaOffchainMessageV0 {
    fn deserialize_reader<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            application_domain: BorshDeserialize::deserialize_reader(reader)?,
            message_format: BorshDeserialize::deserialize_reader(reader)?,
            signers: {
                let signer_count = u8::deserialize_reader(reader)?;
                iter::from_fn(|| Some(BorshDeserialize::deserialize_reader(reader)))
                    .take(signer_count.into())
                    .collect()?
            },
            message: {
                let message_length = u16::deserialize_reader(reader)?;
                let mut m = Vec::with_capacity(message_length.into());
            },
        })
    }
}

#[cfg_attr(
    all(feature = "abi", not(target_arch = "wasm32")),
    serde_as(schemars = true)
)]
#[cfg_attr(
    not(all(feature = "abi", not(target_arch = "wasm32"))),
    serde_as(schemars = false)
)]
#[near(serializers = [json])]
#[autoimpl(Deref using self.payload)]
#[derive(Debug, Clone)]
pub struct SignedSolanaOffchainMessage {
    pub payload: SolanaOffchainMessage,
    #[serde_as(as = "Vec<AsCurve<Ed25519>>")]
    pub signatures: Vec<<Ed25519 as Curve>::Signature>,
}

impl Payload for SignedSolanaOffchainMessage {
    #[inline]
    fn hash(&self) -> CryptoHash {
        self.payload.hash()
    }
}

impl SignedPayload for SignedSolanaOffchainMessage {
    type PublicKey = <Ed25519 as Curve>::PublicKey;

    fn verify(&self) -> Option<Self::PublicKey> {
        // only single signer supported
        let [signer] = self.payload.signers().try_into().ok()?;
        let [signature] = self.signatures.as_slice().try_into().ok()?;

        env::ed25519_verify(&signature, &borsh::to_vec(&self.payload).ok()?, &signer)
            .then_some(signer)
    }
}

#[near(serializers = [borsh, json])]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum MessageFormat {
    RestrictedAscii,
    LimitedUtf8,
    ExtendedUtf8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify() {}
}
