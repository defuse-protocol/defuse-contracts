use impl_tools::autoimpl;
use near_sdk::{
    near,
    serde::de::{self, DeserializeOwned},
    serde_json, AccountId,
};

use crate::{nep413::Nep413Payload, utils::Deadline};

use super::DefusePayload;

#[near(serializers = [json])]
#[autoimpl(Deref using self.message)]
#[autoimpl(DerefMut using self.message)]
#[derive(Debug, Clone)]
pub struct Nep413DefuseMessage<T> {
    pub signer_id: AccountId,

    pub deadline: Deadline,

    #[serde(flatten)]
    pub message: T,
}

impl<T> TryFrom<Nep413Payload> for DefusePayload<T>
where
    T: DeserializeOwned,
{
    type Error = serde_json::Error;

    fn try_from(
        Nep413Payload {
            message,
            nonce,
            recipient,
            callback_url: _,
        }: Nep413Payload,
    ) -> Result<Self, Self::Error> {
        let Nep413DefuseMessage {
            signer_id,
            deadline,
            message,
        } = serde_json::from_str(&message)?;
        Ok(DefusePayload {
            signer_id,
            verifying_contract: recipient.parse().map_err(|_| {
                de::Error::invalid_value(de::Unexpected::Str(&recipient), &"AccountId")
            })?,
            deadline,
            nonce,
            message,
        })
    }
}
