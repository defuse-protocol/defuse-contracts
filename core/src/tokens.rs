use core::{
    fmt::{self, Debug, Display},
    str::FromStr,
};
use std::collections::{btree_map, BTreeMap, HashMap};

use defuse_map_utils::{
    cleanup::{DefaultEntry, DefaultMap},
    IterableMap, Map,
};
use defuse_num_utils::{CheckedAdd, CheckedSub};
use impl_tools::autoimpl;
use near_account_id::ParseAccountError;
use near_sdk::{near, AccountId, AccountIdRef};
use serde_with::{serde_as, DeserializeFromStr, DisplayFromStr, SerializeDisplay};
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
        defuse_nep245::TokenId,
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
        // TODO: refactor
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

// TODO
pub enum TokenIdRef<'a> {
    Nep141(
        /// Contract
        &'a AccountIdRef,
    ),
    Nep171(
        /// Contract
        &'a AccountIdRef,
        /// Token ID
        &'a str,
    ),
    Nep245(
        /// Contract
        &'a AccountIdRef,
        /// Token ID
        &'a str,
    ),
}

#[derive(Debug, ThisError)]
pub enum ParseTokenIdError {
    #[error("AccountId: {0}")]
    AccountId(#[from] ParseAccountError),
    #[error(transparent)]
    ParseError(#[from] strum::ParseError),
}

#[near(serializers = [borsh, json])]
#[autoimpl(Deref using self.0)]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TokenAmounts<T>(T);

impl<T> TokenAmounts<T> {
    #[inline]
    pub const fn new(map: T) -> Self {
        Self(map)
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> TokenAmounts<T>
where
    T: DefaultMap<K = TokenId>,
    T::V: Copy,
{
    #[inline]
    pub fn balance_of(&self, token_id: &TokenId) -> T::V {
        self.0.get(token_id).copied().unwrap_or_default()
    }

    #[inline]
    pub fn deposit(&mut self, token_id: TokenId, amount: u128) -> Option<T::V>
    where
        T::V: CheckedAdd<u128>,
    {
        self.checked_apply(token_id, |a| a.checked_add(amount))
    }

    #[inline]
    pub fn with_deposit(mut self, token_id: TokenId, amount: u128) -> Option<Self>
    where
        T::V: CheckedAdd<u128>,
    {
        self.deposit(token_id, amount)?;
        Some(self)
    }

    #[inline]
    pub fn with_deposit_many(
        self,
        token_amounts: impl IntoIterator<Item = (TokenId, u128)>,
    ) -> Option<Self>
    where
        T::V: CheckedAdd<u128>,
    {
        token_amounts
            .into_iter()
            .try_fold(self, |amounts, (token_id, amount)| {
                amounts.with_deposit(token_id, amount)
            })
    }

    #[inline]
    pub fn withdraw(&mut self, token_id: TokenId, amount: u128) -> Option<T::V>
    where
        T::V: CheckedSub<u128>,
    {
        self.checked_apply(token_id, |a| a.checked_sub(amount))
    }

    #[inline]
    pub fn with_withdraw(mut self, token_id: TokenId, amount: u128) -> Option<Self>
    where
        T::V: CheckedSub<u128>,
    {
        self.withdraw(token_id, amount)?;
        Some(self)
    }

    #[inline]
    pub fn with_withdraw_many(
        self,
        token_amounts: impl IntoIterator<Item = (TokenId, u128)>,
    ) -> Option<Self>
    where
        T::V: CheckedSub<u128>,
    {
        token_amounts
            .into_iter()
            .try_fold(self, |amounts, (token_id, amount)| {
                amounts.with_withdraw(token_id, amount)
            })
    }

    #[inline]
    pub fn add_delta(&mut self, token_id: TokenId, delta: i128) -> Option<T::V>
    where
        T::V: CheckedAdd<i128>,
    {
        self.checked_apply(token_id, |a| a.checked_add(delta))
    }

    #[inline]
    pub fn with_add_delta(mut self, token_id: TokenId, delta: i128) -> Option<Self>
    where
        T::V: CheckedAdd<i128>,
    {
        self.add_delta(token_id, delta)?;
        Some(self)
    }

    #[inline]
    pub fn with_add_deltas(
        self,
        token_amounts: impl IntoIterator<Item = (TokenId, i128)>,
    ) -> Option<Self>
    where
        T::V: CheckedAdd<i128>,
    {
        token_amounts
            .into_iter()
            .try_fold(self, |amounts, (token_id, delta)| {
                amounts.with_add_delta(token_id, delta)
            })
    }

    #[inline]
    fn checked_apply(
        &mut self,
        token_id: TokenId,
        f: impl FnOnce(T::V) -> Option<T::V>,
    ) -> Option<T::V> {
        let mut a = self.0.entry_or_default(token_id);
        *a = f(*a)?;
        Some(*a)
    }
}

impl<T> IntoIterator for TokenAmounts<T>
where
    T: IntoIterator,
{
    type Item = T::Item;

    type IntoIter = T::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.into_inner().into_iter()
    }
}

impl<'a, T> IntoIterator for &'a TokenAmounts<T>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;

    type IntoIter = <&'a T as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> TokenAmounts<T>
where
    T: IterableMap,
{
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

// impl<'a, K, VE, OE> TokenBalance for DefaultEntry<'a, K, u128, VE, OE>
// where
//     K: Ord,
//     VE: VacantEntry<'a, K, u128>,
//     OE: OccupiedEntry<K, u128>,
// {
//     #[inline]
//     fn add_delta(&mut self, delta: i128) -> Option<u128> {
//         **self = self.checked_add_signed(delta)?;
//         Some(**self)
//     }

//     #[inline]
//     fn deposit(&mut self, amount: u128) -> Option<u128> {
//         **self = self.checked_add(amount)?;
//         Some(**self)
//     }

//     #[inline]
//     fn withdraw(&mut self, amount: u128) -> Option<u128> {
//         **self = self.checked_sub(amount)?;
//         Some(**self)
//     }
// }

// impl TokenBalancesView for TokenAmounts<u128> {
//     #[inline]
//     fn balance_of(&self, token_id: &TokenId) -> u128 {
//         self.0.get(token_id).cloned().unwrap_or_default()
//     }
// }

// impl TokenBalances for TokenAmounts<u128> {
//     type Balance<'a> = DefaultEntry<
//         'a,
//         TokenId,
//         u128,
//         btree_map::VacantEntry<'a, TokenId, u128>, btree_map::OccupiedEntry<'a, TokenId, u128>,
//     > where
//         Self: 'a;

//     fn balance_of_mut(&mut self, token_id: TokenId) -> Self::Balance<'_> {
//         self.0.entry_or_default(token_id)
//     }
// }

// #[cfg(all(feature = "abi", not(target_arch = "wasm32")))]
// mod abi {
//     use super::*;

//     use near_sdk::schemars::{
//         gen::SchemaGenerator,
//         schema::{InstanceType, Schema, SchemaObject},
//         JsonSchema,
//     };

//     impl JsonSchema for TokenId {
//         fn schema_name() -> String {
//             stringify!(TokenId).to_string()
//         }

//         fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
//             SchemaObject {
//                 instance_type: Some(InstanceType::String.into()),
//                 extensions: [(
//                     "examples",
//                     [
//                         Self::Nep141("ft.near".parse().unwrap()),
//                         Self::Nep171("nft.near".parse().unwrap(), "token_id1".to_string()),
//                         Self::Nep245("mt.near".parse().unwrap(), "token_id1".to_string()),
//                     ]
//                     .map(|s| s.to_string())
//                     .to_vec()
//                     .into(),
//                 )]
//                 .into_iter()
//                 .map(|(k, v)| (k.to_string(), v))
//                 .collect(),
//                 ..Default::default()
//             }
//             .into()
//         }
//     }
// }

// #[cfg_attr(
//     all(feature = "abi", not(target_arch = "wasm32")),
//     serde_as(schemars = true)
// )]
// #[cfg_attr(
//     not(all(feature = "abi", not(target_arch = "wasm32"))),
//     serde_as(schemars = false)
// )]
// #[near(serializers = [borsh, json])]
// #[serde(bound(serialize = "T: Display", deserialize = "T: FromStr<Err: Display>"))]
// #[autoimpl(Deref using self.0)]
// #[autoimpl(Default)]
// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct TokenAmounts<T>(
//     /// [`BTreeMap`] ensures deterministic order
//     #[serde_as(as = "BTreeMap<_, DisplayFromStr>")]
//     BTreeMap<TokenId, T>,
// );

// impl<A> TokenAmounts<A> {
//     #[inline]
//     pub fn checked_add<T>(&mut self, token_id: TokenId, amount: T) -> Option<A>
//     where
//         A: CheckedAdd<T> + Default + Eq + Copy,
//     {
//         self.checked_apply(token_id, |a| a.checked_add(amount))
//     }

//     #[inline]
//     pub fn checked_sub<T>(&mut self, token_id: TokenId, amount: T) -> Option<A>
//     where
//         A: CheckedSub<T> + Default + Eq + Copy,
//     {
//         self.checked_apply(token_id, |a| a.checked_sub(amount))
//     }

//     #[inline]
//     fn checked_apply(&mut self, token_id: TokenId, f: impl FnOnce(A) -> Option<A>) -> Option<A>
//     where
//         A: Default + Eq + Copy,
//     {
//         let mut d = self.0.entry_or_default(token_id);
//         *d = f(*d)?;
//         Some(*d)
//     }

//     #[inline]
//     pub fn extend<T>(
//         &mut self,
//         amounts: impl IntoIterator<Item = (TokenId, T)>,
//     ) -> Option<&mut Self>
//     where
//         A: CheckedAdd<T> + Default + Eq + Copy,
//     {
//         for (token_id, amount) in amounts {
//             self.checked_add(token_id, amount)?;
//         }
//         Some(self)
//     }

//     #[inline]
//     pub fn with_extend<T>(mut self, amounts: impl IntoIterator<Item = (TokenId, T)>) -> Option<Self>
//     where
//         A: CheckedAdd<T> + Default + Eq + Copy,
//     {
//         self.extend(amounts)?;
//         Some(self)
//     }

//     #[inline]
//     pub fn from_iter<T>(iter: impl IntoIterator<Item = (TokenId, T)>) -> Option<Self>
//     where
//         A: CheckedAdd<T> + Default + Eq + Copy,
//     {
//         iter.into_iter()
//             .try_fold(Self::default(), |mut amounts, (token_id, amount)| {
//                 amounts.checked_add(token_id, amount).map(|_| amounts)
//             })
//     }

//     #[inline]
//     pub fn into_tokens(self) -> impl Iterator<Item = TokenId> {
//         self.0.into_keys()
//     }

//     #[inline]
//     pub fn into_amounts(self) -> impl Iterator<Item = A> {
//         self.0.into_values()
//     }

//     #[inline]
//     pub fn is_empty(&self) -> bool {
//         self.0.is_empty()
//     }
// }

// impl<A> IntoIterator for TokenAmounts<A> {
//     type Item = (TokenId, A);

//     type IntoIter = btree_map::IntoIter<TokenId, A>;

//     #[inline]
//     fn into_iter(self) -> Self::IntoIter {
//         self.0.into_iter()
//     }
// }

// impl<'a, T> IntoIterator for &'a TokenAmounts<T> {
//     type Item = (&'a TokenId, &'a T);

//     type IntoIter = btree_map::Iter<'a, TokenId, T>;

//     #[inline]
//     fn into_iter(self) -> Self::IntoIter {
//         self.0.iter()
//     }
// }

// #[cfg(test)]
// mod tests {

//     use super::*;

//     #[test]
//     fn invariant() {
//         let [t1, t2] = ["t1.near", "t2.near"].map(|t| TokenId::Nep141(t.parse().unwrap()));

//         assert!(TokenAmounts::<()>::default().is_empty());
//         assert!(TokenAmounts::<i32>::default()
//             .with_extend([(t1.clone(), 0)])
//             .unwrap()
//             .is_empty());

//         assert!(!TokenAmounts::<i32>::default()
//             .with_extend([(t1.clone(), 1)])
//             .unwrap()
//             .is_empty());

//         assert!(!TokenAmounts::<i32>::default()
//             .with_extend([(t1.clone(), -1)])
//             .unwrap()
//             .is_empty());

//         assert!(TokenAmounts::<i32>::default()
//             .with_extend([(t1.clone(), 1), (t1.clone(), -1)])
//             .unwrap()
//             .is_empty());

//         assert!(!TokenAmounts::<i32>::default()
//             .with_extend([(t1.clone(), 1), (t1.clone(), -1), (t2.clone(), -1)])
//             .unwrap()
//             .is_empty());

//         assert!(TokenAmounts::<i32>::default()
//             .with_extend([
//                 (t1.clone(), 1),
//                 (t1.clone(), -1),
//                 (t2.clone(), -1),
//                 (t2.clone(), 1)
//             ])
//             .unwrap()
//             .is_empty());
//     }
// }
