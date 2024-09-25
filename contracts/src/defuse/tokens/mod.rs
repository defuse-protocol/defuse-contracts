pub mod nep141;

use core::{
    fmt::{self, Debug, Display},
    str::FromStr,
};

use near_account_id::ParseAccountError;
use near_sdk::{near, AccountId};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use strum::EnumString;
use thiserror::Error as ThisError;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeDisplay, DeserializeFromStr)]
#[near(serializers = [borsh])]
pub enum TokenId {
    Nep141(AccountId),
    Nep171(
        AccountId,
        near_contract_standards::non_fungible_token::TokenId,
    ),
    Defuse(String),
}

impl TokenId {
    #[must_use]
    #[inline]
    pub const fn typ(&self) -> TokenIdType {
        match self {
            Self::Nep141(_) => TokenIdType::Nep141,
            Self::Nep171(_, _) => TokenIdType::Nep171,
            Self::Defuse(_) => TokenIdType::Defuse,
        }
    }
}

impl Debug for TokenId {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nep141(contract_id) => {
                write!(f, "{}:{}", TokenIdType::Nep141, contract_id)
            }
            Self::Nep171(contract_id, token_id) => {
                write!(f, "{}:{}:{}", TokenIdType::Nep171, contract_id, token_id)
            }
            Self::Defuse(defuse_id) => {
                write!(f, "{}:{}", TokenIdType::Defuse, defuse_id)
            }
        }
    }
}

impl Display for TokenId {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl FromStr for TokenId {
    type Err = ParseTokenIdError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (typ, data) = s
            .split_once(':')
            .ok_or(strum::ParseError::VariantNotFound)?;
        Ok(match typ.parse()? {
            TokenIdType::Nep141 => Self::Nep141(data.parse()?),
            TokenIdType::Nep171 => {
                let (contract_id, token_id) = data
                    .split_once(':')
                    .ok_or(strum::ParseError::VariantNotFound)?;
                Self::Nep171(contract_id.parse()?, token_id.to_string())
            }
            TokenIdType::Defuse => Self::Defuse(data.to_string()),
        })
    }
}

#[derive(strum::Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum TokenIdType {
    Nep141,
    Nep171,
    Defuse,
}

#[derive(Debug, ThisError)]
pub enum ParseTokenIdError {
    #[error("AccountId: {0}")]
    AccountId(#[from] ParseAccountError),
    #[error(transparent)]
    ParseError(#[from] strum::ParseError),
}
