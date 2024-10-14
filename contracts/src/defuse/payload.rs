use core::convert::Infallible;

use derive_more::derive::From;
use impl_tools::autoimpl;
use near_sdk::{borsh::BorshSerialize, near, AccountId};

use crate::{
    crypto::{Payload, SignedPayload},
    nep413::Nep413Payload,
};

pub type SignedDefusePayload<T> = SignedPayload<MultiStandardPayload<SignerPayload<T>>>;
pub type DefusePayload<T> = Nep413Payload<SignerPayload<T>>;

#[near(serializers = [borsh, json])]
#[autoimpl(Deref using self.payload)]
#[autoimpl(DerefMut using self.payload)]
#[derive(Debug, Clone)]
pub struct SignerPayload<T> {
    pub signer_id: AccountId,

    #[serde(flatten)]
    pub payload: T,
}

#[near(serializers = [borsh, json])]
#[serde(tag = "standard", rename_all = "snake_case")]
#[derive(Debug, Clone, From)]
pub enum MultiStandardPayload<T> {
    Nep413(Nep413Payload<T>),
}

impl<T> Payload for MultiStandardPayload<T>
where
    T: BorshSerialize,
{
    #[inline]
    fn hash(&self) -> [u8; 32] {
        match self {
            Self::Nep413(payload) => payload.hash(),
        }
    }
}

pub trait ValidatePayloadAs<T> {
    type Error;

    /// Validates self and extracts `T`
    fn validate_as(self) -> Result<T, Self::Error>;
}

impl<T> ValidatePayloadAs<T> for T {
    type Error = Infallible;

    #[inline]
    fn validate_as(self) -> Result<Self, Self::Error> {
        Ok(self)
    }
}

impl<T> ValidatePayloadAs<Nep413Payload<T>> for MultiStandardPayload<T> {
    type Error = Infallible;

    #[inline]
    fn validate_as(self) -> Result<Nep413Payload<T>, Self::Error> {
        match self {
            Self::Nep413(payload) => Ok(payload),
        }
    }
}
