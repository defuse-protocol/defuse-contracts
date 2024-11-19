use near_sdk::serde::de::DeserializeOwned;

use crate::solana::{OffchainMessage, OffchainMessageV0};

use super::DefusePayload;

impl<T> TryFrom<OffchainMessage> for DefusePayload<T>
where
    T: DeserializeOwned,
{
    type Error = DefuseSolanaOffchainMessageParseError;

    fn try_from(message: OffchainMessage) -> Result<Self, Self::Error> {
        match message {
            OffchainMessage::V0(OffchainMessageV0 {
                application_domain,
                message_format,
                signers,
                message,
            }) => {
                todo!()
                // let [signer] = signers
                //     .try_into()
                //     .map_err(|_| DefuseSolanaOffchainMessageParseError::InvalidSigner)?;
                // Ok(Self {
                //     signer_id: signer.to_implicit_account(),
                //     verifying_contract: todo!(),
                //     deadline: todo!(),
                //     nonce: todo!(),
                //     message: todo!(),
                // })
            }
            _ => Err(DefuseSolanaOffchainMessageParseError::UnsupportedVersion),
        }
    }
}

pub enum DefuseSolanaOffchainMessageParseError {
    UnsupportedVersion,
    InvalidSigner,
}
