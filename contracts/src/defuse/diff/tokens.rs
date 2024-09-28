use std::collections::btree_map::{self, BTreeMap};

use impl_tools::autoimpl;
use near_sdk::near;
use serde_with::{serde_as, DisplayFromStr};

use crate::{
    defuse::{tokens::TokenId, DefuseError, Result},
    utils::cleanup::DefaultMap,
};

#[derive(Debug, Clone)]
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
pub struct TokenDeltas<T: Ord = TokenId>(
    /// [`BTreeMap`] ensures deterministic order
    #[serde_as(as = "BTreeMap<_, DisplayFromStr>")]
    // HACK
    #[serde(bound(
        serialize = "T: ::near_sdk::serde::Serialize",
        deserialize = "T: ::near_sdk::serde::Deserialize<'de>"
    ))]
    BTreeMap<T, i128>,
);

impl<T: Ord> TokenDeltas<T> {
    #[inline]
    pub fn append<I>(&mut self, iter: I) -> Result<()>
    where
        I: IntoIterator<Item = (T, i128)>,
    {
        for (token_id, delta) in iter {
            self.add_delta(token_id, delta)?;
        }
        Ok(())
    }

    #[inline]
    pub fn add_delta(&mut self, token_id: T, delta: i128) -> Result<i128> {
        let mut d = self.0.entry_or_default(token_id);
        *d = d.checked_add(delta).ok_or(DefuseError::BalanceOverflow)?;
        Ok(*d)
    }

    #[inline]
    pub fn with_add_delta(mut self, token_id: T, delta: i128) -> Result<Self> {
        self.add_delta(token_id, delta)?;
        Ok(self)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl IntoIterator for TokenDeltas {
    type Item = (TokenId, i128);

    type IntoIter = btree_map::IntoIter<TokenId, i128>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a TokenDeltas {
    type Item = (&'a TokenId, &'a i128);

    type IntoIter = btree_map::Iter<'a, TokenId, i128>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn invariant() {
        let [t1, t2] = ["t1.near", "t2.near"].map(|t| TokenId::Nep141(t.parse().unwrap()));

        assert!(TokenDeltas::<()>::default().is_empty());
        assert!(TokenDeltas::default()
            .with_add_delta(t1.clone(), 0)
            .unwrap()
            .is_empty());

        assert!(!TokenDeltas::default()
            .with_add_delta(t1.clone(), 1)
            .unwrap()
            .is_empty());

        assert!(!TokenDeltas::default()
            .with_add_delta(t1.clone(), -1)
            .unwrap()
            .is_empty());

        assert!(TokenDeltas::default()
            .with_add_delta(t1.clone(), 1)
            .unwrap()
            .with_add_delta(t1.clone(), -1)
            .unwrap()
            .is_empty());

        assert!(!TokenDeltas::default()
            .with_add_delta(t1.clone(), 1)
            .unwrap()
            .with_add_delta(t1.clone(), -1)
            .unwrap()
            .with_add_delta(t2.clone(), -1)
            .unwrap()
            .is_empty());

        assert!(TokenDeltas::default()
            .with_add_delta(t1.clone(), 1)
            .unwrap()
            .with_add_delta(t2.clone(), -1)
            .unwrap()
            .with_add_delta(t1.clone(), -1)
            .unwrap()
            .with_add_delta(t2.clone(), 1)
            .unwrap()
            .is_empty());
    }
}
