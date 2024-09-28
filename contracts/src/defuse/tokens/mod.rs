pub mod nep141;
pub mod nep171;
pub mod nep245;

use core::{
    fmt::{self, Debug, Display},
    str::FromStr,
};

use near_account_id::ParseAccountError;
use near_sdk::{near, AccountId};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use strum::{EnumDiscriminants, EnumString};
use thiserror::Error as ThisError;

#[derive(
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    EnumDiscriminants,
    SerializeDisplay,
    DeserializeFromStr,
)]
#[strum_discriminants(
    name(TokenIdType),
    derive(strum::Display, EnumString),
    strum(serialize_all = "snake_case")
)]
#[near(serializers = [borsh])]
pub enum TokenId {
    Nep141(
        /// Contract
        AccountId,
    ),
    Nep171(
        /// Contract
        AccountId,
        /// Token ID
        near_contract_standards::non_fungible_token::TokenId,
    ),
    Nep245(
        /// Contract
        AccountId,
        /// Token ID
        crate::nep245::TokenId,
    ),
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
            Self::Nep245(contract_id, token_id) => {
                write!(f, "{}:{}:{}", TokenIdType::Nep245, contract_id, token_id)
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
            TokenIdType::Nep245 => {
                let (contract_id, token_id) = data
                    .split_once(':')
                    .ok_or(strum::ParseError::VariantNotFound)?;
                Self::Nep245(contract_id.parse()?, token_id.to_string())
            }
        })
    }
}

#[derive(Debug, ThisError)]
pub enum ParseTokenIdError {
    #[error("AccountId: {0}")]
    AccountId(#[from] ParseAccountError),
    #[error(transparent)]
    ParseError(#[from] strum::ParseError),
}

#[cfg(all(feature = "abi", not(target_arch = "wasm32")))]
mod abi {
    use super::*;

    use near_sdk::schemars::{gen::SchemaGenerator, schema::Schema, JsonSchema};

    impl JsonSchema for TokenId {
        fn schema_name() -> String {
            String::schema_name()
        }

        fn json_schema(gen: &mut SchemaGenerator) -> Schema {
            String::json_schema(gen)
        }

        fn is_referenceable() -> bool {
            false
        }
    }
}
