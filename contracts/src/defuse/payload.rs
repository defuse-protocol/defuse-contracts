use core::convert::Infallible;
use std::collections::HashMap;

use near_sdk::{borsh::BorshSerialize, near, AccountId};

use crate::{
    crypto::{Payload, SignedPayload},
    nep413::Nep413Payload,
};

pub type SignedPayloads<T> = HashMap<
    // Signer account
    AccountId,
    // Payloads signed by the account
    Vec<SignedPayload<MultiStandardPayload<T>>>,
>;

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

impl<T> From<Nep413Payload<T>> for MultiStandardPayload<T> {
    #[inline]
    fn from(value: Nep413Payload<T>) -> Self {
        Self::Nep413(value)
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
