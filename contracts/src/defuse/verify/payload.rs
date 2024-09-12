use std::convert::Infallible;

use near_sdk::{borsh::BorshSerialize, near};

use crate::{crypto::Payload, nep413::Nep413Payload};

pub trait ValidatePayloadAs<T> {
    type Error;

    fn validate_as(self) -> Result<T, Self::Error>;
}

impl<T> ValidatePayloadAs<T> for T {
    type Error = Infallible;

    #[inline]
    fn validate_as(self) -> Result<Self, Self::Error> {
        Ok(self)
    }
}

#[derive(Debug, Clone)]
#[near(serializers = [borsh, json])]
#[serde(tag = "standard", rename_all = "snake_case")]
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

impl<T> ValidatePayloadAs<Nep413Payload<T>> for MultiStandardPayload<T> {
    type Error = Infallible;

    #[inline]
    fn validate_as(self) -> Result<Nep413Payload<T>, Self::Error> {
        match self {
            Self::Nep413(payload) => Ok(payload),
        }
    }
}

impl<T> From<Nep413Payload<T>> for MultiStandardPayload<T> {
    #[inline]
    fn from(value: Nep413Payload<T>) -> Self {
        Self::Nep413(value)
    }
}
