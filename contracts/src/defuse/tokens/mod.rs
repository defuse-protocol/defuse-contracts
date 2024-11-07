pub mod nep141;
pub mod nep171;
pub mod nep245;

use core::{
    fmt::{self, Debug, Display},
    str::FromStr,
};
use std::collections::{btree_map, BTreeMap};

use impl_tools::autoimpl;
use near_account_id::ParseAccountError;
use near_sdk::{near, serde_json, AccountId};
use serde_with::{serde_as, DeserializeFromStr, DisplayFromStr, SerializeDisplay};
use strum::{EnumDiscriminants, EnumString};
use thiserror::Error as ThisError;

use crate::{
    crypto::SignedPayload,
    utils::{
        cleanup::DefaultMap,
        integer::{CheckedAdd, CheckedSub},
        UnwrapOrPanicError,
    },
};

use super::{payload::multi::MultiStandardPayload, DefuseError, Result};

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

    use near_sdk::schemars::{
        gen::SchemaGenerator,
        schema::{InstanceType, Schema, SchemaObject},
        JsonSchema,
    };

    impl JsonSchema for TokenId {
        fn schema_name() -> String {
            stringify!(TokenId).to_string()
        }

        fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
            SchemaObject {
                instance_type: Some(InstanceType::String.into()),
                extensions: [(
                    "examples",
                    [
                        Self::Nep141("ft.near".parse().unwrap()),
                        Self::Nep171("nft.near".parse().unwrap(), "token_id1".to_string()),
                        Self::Nep245("mt.near".parse().unwrap(), "token_id1".to_string()),
                    ]
                    .map(|s| s.to_string())
                    .to_vec()
                    .into(),
                )]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
                ..Default::default()
            }
            .into()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[autoimpl(Default)]
#[autoimpl(Deref using self.0)]
#[cfg_attr(
    all(feature = "abi", not(target_arch = "wasm32")),
    serde_as(schemars = true)
)]
#[cfg_attr(
    not(all(feature = "abi", not(target_arch = "wasm32"))),
    serde_as(schemars = false)
)]
#[near(serializers = [borsh, json])]
#[serde(bound(serialize = "T: Display", deserialize = "T: FromStr<Err: Display>"))] // HACK
pub struct TokenAmounts<T>(
    /// [`BTreeMap`] ensures deterministic order
    #[serde_as(as = "BTreeMap<_, DisplayFromStr>")]
    BTreeMap<TokenId, T>,
);

impl<A> TokenAmounts<A> {
    #[inline]
    pub fn add<T>(&mut self, token_id: TokenId, amount: T) -> Result<A>
    where
        A: CheckedAdd<T> + Default + Eq + Copy,
    {
        self.try_apply(token_id, |a| {
            a.checked_add(amount).ok_or(DefuseError::BalanceOverflow)
        })
    }

    #[inline]
    pub fn sub<T>(&mut self, token_id: TokenId, amount: T) -> Result<A>
    where
        A: CheckedSub<T> + Default + Eq + Copy,
    {
        self.try_apply(token_id, |a| {
            a.checked_sub(amount).ok_or(DefuseError::BalanceOverflow)
        })
    }

    #[inline]
    fn try_apply<E>(&mut self, token_id: TokenId, f: impl FnOnce(A) -> Result<A, E>) -> Result<A, E>
    where
        A: Default + Eq + Copy,
    {
        let mut d = self.0.entry_or_default(token_id);
        *d = f(*d)?;
        Ok(*d)
    }

    #[inline]
    pub fn try_extend<T>(
        &mut self,
        amounts: impl IntoIterator<Item = (TokenId, T)>,
    ) -> Result<&mut Self>
    where
        A: CheckedAdd<T> + Default + Eq + Copy,
    {
        for (token_id, amount) in amounts {
            self.add(token_id, amount)?;
        }
        Ok(self)
    }

    #[inline]
    pub fn with_try_extend<T>(
        mut self,
        amounts: impl IntoIterator<Item = (TokenId, T)>,
    ) -> Result<Self>
    where
        A: CheckedAdd<T> + Default + Eq + Copy,
    {
        self.try_extend(amounts)?;
        Ok(self)
    }

    #[inline]
    pub fn try_from_iter<T>(iter: impl IntoIterator<Item = (TokenId, T)>) -> Result<Self>
    where
        A: CheckedAdd<T> + Default + Eq + Copy,
    {
        iter.into_iter()
            .try_fold(Self::default(), |mut amounts, (token_id, amount)| {
                amounts.add(token_id, amount).map(|_| amounts)
            })
    }

    #[inline]
    pub fn into_tokens(self) -> impl Iterator<Item = TokenId> {
        self.0.into_keys()
    }

    #[inline]
    pub fn into_amounts(self) -> impl Iterator<Item = A> {
        self.0.into_values()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<A> IntoIterator for TokenAmounts<A> {
    type Item = (TokenId, A);

    type IntoIter = btree_map::IntoIter<TokenId, A>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a TokenAmounts<T> {
    type Item = (&'a TokenId, &'a T);

    type IntoIter = btree_map::Iter<'a, TokenId, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[near(serializers = [json])]
#[derive(Debug, Clone)]
pub struct DepositMessage {
    pub receiver_id: AccountId,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub execute_intents: Vec<SignedPayload<MultiStandardPayload>>,

    #[serde(default, skip_serializing_if = "::core::ops::Not::not")]
    pub refund_if_fails: bool,
}

impl DepositMessage {
    #[inline]
    pub const fn new(receiver_id: AccountId) -> Self {
        Self {
            receiver_id,
            execute_intents: Vec::new(),
            refund_if_fails: false,
        }
    }

    #[inline]
    pub fn with_execute_intents(
        mut self,
        intents: impl IntoIterator<Item = SignedPayload<MultiStandardPayload>>,
    ) -> Self {
        self.execute_intents.extend(intents);
        self
    }

    #[inline]
    pub fn with_refund_if_fails(mut self) -> Self {
        self.refund_if_fails = true;
        self
    }
}

impl Display for DepositMessage {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.execute_intents.is_empty() {
            f.write_str(self.receiver_id.as_str())
        } else {
            f.write_str(&serde_json::to_string(self).unwrap_or_panic_display())
        }
    }
}

impl FromStr for DepositMessage {
    type Err = ParseDepositMessageError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('{') {
            serde_json::from_str(s).map_err(Into::into)
        } else {
            s.parse().map(Self::new).map_err(Into::into)
        }
    }
}

#[derive(Debug, ThisError)]
pub enum ParseDepositMessageError {
    #[error(transparent)]
    Account(#[from] ParseAccountError),
    #[error("JSON: {0}")]
    JSON(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn invariant() {
        let [t1, t2] = ["t1.near", "t2.near"].map(|t| TokenId::Nep141(t.parse().unwrap()));

        assert!(TokenAmounts::<()>::default().is_empty());
        assert!(TokenAmounts::<i32>::default()
            .with_try_extend([(t1.clone(), 0)])
            .unwrap()
            .is_empty());

        assert!(!TokenAmounts::<i32>::default()
            .with_try_extend([(t1.clone(), 1)])
            .unwrap()
            .is_empty());

        assert!(!TokenAmounts::<i32>::default()
            .with_try_extend([(t1.clone(), -1)])
            .unwrap()
            .is_empty());

        assert!(TokenAmounts::<i32>::default()
            .with_try_extend([(t1.clone(), 1), (t1.clone(), -1)])
            .unwrap()
            .is_empty());

        assert!(!TokenAmounts::<i32>::default()
            .with_try_extend([(t1.clone(), 1), (t1.clone(), -1), (t2.clone(), -1)])
            .unwrap()
            .is_empty());

        assert!(TokenAmounts::<i32>::default()
            .with_try_extend([
                (t1.clone(), 1),
                (t1.clone(), -1),
                (t2.clone(), -1),
                (t2.clone(), 1)
            ])
            .unwrap()
            .is_empty());
    }
}
