use std::collections::btree_map::{self, BTreeMap, Entry};

use impl_tools::autoimpl;
use near_sdk::near;
use serde_with::{serde_as, DisplayFromStr};

use crate::defuse::{tokens::TokenId, DefuseError};

#[derive(Debug, Clone, Default)]
#[cfg_attr(
    all(feature = "abi", not(target_arch = "wasm32")),
    serde_as(schemars = true)
)]
#[cfg_attr(
    not(all(feature = "abi", not(target_arch = "wasm32"))),
    serde_as(schemars = false)
)]
#[near(serializers = [borsh, json])]
#[autoimpl(Deref using self.0)]
pub struct TokenDeltas(
    /// [`BTreeMap`] ensures deterministic order
    #[serde_as(as = "BTreeMap<_, DisplayFromStr>")]
    BTreeMap<
        TokenId,
        // TODO: i129
        i128,
    >,
);

impl TokenDeltas {
    #[inline]
    pub fn append<I>(&mut self, iter: I) -> Result<(), DefuseError>
    where
        I: IntoIterator<Item = (TokenId, i128)>,
    {
        for (token_id, delta) in iter {
            self.add_delta(token_id, delta)?;
        }
        Ok(())
    }

    #[inline]
    pub fn add_delta(&mut self, token_id: TokenId, delta: i128) -> Result<i128, DefuseError> {
        Ok(match self.0.entry(token_id) {
            Entry::Vacant(_) if delta == 0 => 0,
            Entry::Vacant(entry) => *entry.insert(delta),
            Entry::Occupied(mut entry) => {
                let v = entry.get_mut();
                *v = v.checked_add(delta).ok_or(DefuseError::BalanceOverflow)?;
                if *v == 0 {
                    entry.remove()
                } else {
                    *v
                }
            }
        })
    }

    #[inline]
    pub fn with_add_delta(mut self, token_id: TokenId, delta: i128) -> Result<Self, DefuseError> {
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
