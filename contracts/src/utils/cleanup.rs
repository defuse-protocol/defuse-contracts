use std::{
    collections::{
        btree_map::{self, BTreeMap},
        hash_map::{self, HashMap},
    },
    convert::Infallible,
    hash::Hash,
};

use near_sdk::{
    borsh::{BorshDeserialize, BorshSerialize},
    store::{iterable_map, IterableMap},
};

pub trait DefaultMap<K, V> {
    #[inline]
    fn apply_cleanup_default<T>(&mut self, k: K, f: impl FnOnce(&mut V)) -> V
    where
        V: Default + Eq + Clone,
    {
        self.try_apply_cleanup_default(k, move |v| {
            f(v);
            Ok(())
        })
        .unwrap_or_else(|err: Infallible| match err {})
    }

    fn try_apply_cleanup_default<E>(
        &mut self,
        k: K,
        f: impl FnOnce(&mut V) -> Result<(), E>,
    ) -> Result<V, E>
    where
        V: Default + Eq + Clone;
}

impl<K, V> DefaultMap<K, V> for HashMap<K, V>
where
    K: Hash + Eq,
{
    fn try_apply_cleanup_default<E>(
        &mut self,
        k: K,
        f: impl FnOnce(&mut V) -> Result<(), E>,
    ) -> Result<V, E>
    where
        V: Default + Eq + Clone,
    {
        Ok(match self.entry(k) {
            hash_map::Entry::Occupied(mut entry) => {
                let v = entry.get_mut();
                f(v)?;
                if v == &Default::default() {
                    entry.remove()
                } else {
                    v.clone()
                }
            }
            hash_map::Entry::Vacant(entry) => {
                let mut v = Default::default();
                f(&mut v)?;
                if v != Default::default() {
                    entry.insert(v).clone()
                } else {
                    v.clone()
                }
            }
        })
    }
}

impl<K, V> DefaultMap<K, V> for BTreeMap<K, V>
where
    K: Hash + Ord,
{
    fn try_apply_cleanup_default<E>(
        &mut self,
        k: K,
        f: impl FnOnce(&mut V) -> Result<(), E>,
    ) -> Result<V, E>
    where
        V: Default + Eq + Clone,
    {
        Ok(match self.entry(k) {
            btree_map::Entry::Occupied(mut entry) => {
                let v = entry.get_mut();
                f(v)?;
                if v == &Default::default() {
                    entry.remove()
                } else {
                    v.clone()
                }
            }
            btree_map::Entry::Vacant(entry) => {
                let mut v = Default::default();
                f(&mut v)?;
                if v != Default::default() {
                    entry.insert(v).clone()
                } else {
                    v.clone()
                }
            }
        })
    }
}

impl<K, V> DefaultMap<K, V> for IterableMap<K, V>
where
    K: Ord + Clone + BorshSerialize + BorshDeserialize,
    V: BorshSerialize + BorshDeserialize,
{
    fn try_apply_cleanup_default<E>(
        &mut self,
        k: K,
        f: impl FnOnce(&mut V) -> Result<(), E>,
    ) -> Result<V, E>
    where
        V: Default + Eq + Clone,
    {
        Ok(match self.entry(k) {
            iterable_map::Entry::Occupied(mut entry) => {
                let v = entry.get_mut();
                f(v)?;
                if v == &Default::default() {
                    entry.remove()
                } else {
                    v.clone()
                }
            }
            iterable_map::Entry::Vacant(entry) => {
                let mut v = Default::default();
                f(&mut v)?;
                if v != Default::default() {
                    entry.insert(v).clone()
                } else {
                    v.clone()
                }
            }
        })
    }
}
