use core::{
    convert::Infallible,
    fmt::{self, Display},
    str::FromStr,
};

use derive_more::derive::From;
use impl_tools::autoimpl;
use near_sdk::{
    near,
    serde::{de::DeserializeOwned, Serialize},
    serde_json, AccountId, CryptoHash,
};

use crate::{
    crypto::{Payload, SignedPayload},
    nep413::Nep413Payload,
    utils::Deadline,
};

pub type SignedDefuseMessage<T> = SignedPayload<MultiStandardPayload<DefuseMessage<T>>>;

#[near(serializers = [borsh, json])]
#[autoimpl(Deref using self.message)]
#[autoimpl(DerefMut using self.message)]
#[derive(Debug, Clone)]
pub struct DefuseMessage<T> {
    pub signer_id: AccountId,

    pub deadline: Deadline,

    #[serde(flatten)]
    pub message: T,
}

impl<T> Display for DefuseMessage<T>
where
    T: Serialize,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = if f.alternate() {
            serde_json::to_string_pretty(self)
        } else {
            serde_json::to_string(self)
        }
        .map_err(|_| fmt::Error)?;

        f.write_str(&s)
    }
}

impl<T> FromStr for DefuseMessage<T>
where
    T: DeserializeOwned,
{
    type Err = serde_json::Error;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[near(serializers = [borsh, json])]
#[serde(tag = "standard", rename_all = "snake_case")]
#[derive(Debug, Clone, From)]
pub enum MultiStandardPayload<T> {
    Nep413(
        #[borsh(bound(serialize = "T: Display", deserialize = "T: FromStr<Err: Display>"))]
        #[serde(bound(serialize = "T: Display", deserialize = "T: FromStr<Err: Display>"))]
        Nep413Payload<T>,
    ),
}

impl<T> Payload for MultiStandardPayload<T>
where
    T: Display,
{
    #[inline]
    fn hash(&self) -> CryptoHash {
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
