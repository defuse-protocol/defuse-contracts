use core::{
    hash::Hash,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};
use std::collections::{
    btree_map::{self, BTreeMap},
    hash_map::{self, HashMap},
};

use near_sdk::{
    borsh::{BorshDeserialize, BorshSerialize},
    store::{iterable_map, key::ToKey, IterableMap},
};

/// A mapping where non-existent entries considered to be [`Default`] values
pub trait DefaultMap<K, V>
where
    V: Default + Eq,
{
    type VacantEntry<'a>: VacantEntry<V>
    where
        Self: 'a;
    type OccupiedEntry<'a>: OccupiedEntry<V>
    where
        Self: 'a;

    /// Get an entry at given key or [`Default`] value if the key doesn't exist.
    ///
    /// The returned entry will automatically be removed from the map if it becomes
    /// equal to [`Default`] after modifications.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use std::collections::HashMap;
    /// # use defuse_contracts::utils::cleanup::DefaultMap;
    /// let mut m: HashMap<&str, i32> = HashMap::new();
    /// *m.entry_or_default("a") += 1;
    /// assert_eq!(m.get("a"), Some(&1));
    /// *m.entry_or_default("a") -= 1;
    /// assert_eq!(m.get("a"), None);
    /// ```
    fn entry_or_default(
        &mut self,
        key: K,
    ) -> DefaultEntry<V, Self::VacantEntry<'_>, Self::OccupiedEntry<'_>>;
}

pub enum DefaultEntry<T, V, O>
where
    T: Default + Eq,
    V: VacantEntry<T>,
    O: OccupiedEntry<T>,
{
    Vacant(DefaultVacantEntry<T, V>),
    Occupied(DefaultOccupiedEntry<T, O>),
}

impl<T, V, O> From<DefaultVacantEntry<T, V>> for DefaultEntry<T, V, O>
where
    T: Default + Eq,
    V: VacantEntry<T>,
    O: OccupiedEntry<T>,
{
    #[inline]
    fn from(entry: DefaultVacantEntry<T, V>) -> Self {
        Self::Vacant(entry)
    }
}

impl<T, V, O> From<DefaultOccupiedEntry<T, O>> for DefaultEntry<T, V, O>
where
    T: Default + Eq,
    V: VacantEntry<T>,
    O: OccupiedEntry<T>,
{
    #[inline]
    fn from(entry: DefaultOccupiedEntry<T, O>) -> Self {
        Self::Occupied(entry)
    }
}

pub struct DefaultVacantEntry<T, V>(Option<(T, V)>)
where
    T: Default + Eq,
    V: VacantEntry<T>;

impl<T, V> DefaultVacantEntry<T, V>
where
    T: Default + Eq,
    V: VacantEntry<T>,
{
    #[inline]
    pub fn new(entry: V) -> Self {
        Self(Some((T::default(), entry)))
    }
}

impl<T, V> Deref for DefaultVacantEntry<T, V>
where
    T: Default + Eq,
    V: VacantEntry<T>,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0.as_ref().unwrap_or_else(|| unreachable!()).0
    }
}

impl<T, V> DerefMut for DefaultVacantEntry<T, V>
where
    T: Default + Eq,
    V: VacantEntry<T>,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.as_mut().unwrap_or_else(|| unreachable!()).0
    }
}

impl<T, V> Drop for DefaultVacantEntry<T, V>
where
    T: Default + Eq,
    V: VacantEntry<T>,
{
    #[inline]
    fn drop(&mut self) {
        let (v, entry) = self.0.take().unwrap_or_else(|| unreachable!());
        if v != Default::default() {
            entry.insert(v);
        }
    }
}

pub struct DefaultOccupiedEntry<T, O>(Option<O>, PhantomData<T>)
where
    T: Default + Eq,
    O: OccupiedEntry<T>;

impl<T, O> DefaultOccupiedEntry<T, O>
where
    T: Default + Eq,
    O: OccupiedEntry<T>,
{
    #[inline]
    pub fn new(entry: O) -> Self {
        Self(Some(entry), PhantomData)
    }
}

impl<T, O> Deref for DefaultOccupiedEntry<T, O>
where
    T: Default + Eq,
    O: OccupiedEntry<T>,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap_or_else(|| unreachable!()).get()
    }
}

impl<T, O> DerefMut for DefaultOccupiedEntry<T, O>
where
    T: Default + Eq,
    O: OccupiedEntry<T>,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap_or_else(|| unreachable!()).get_mut()
    }
}

impl<T, O> Drop for DefaultOccupiedEntry<T, O>
where
    T: Default + Eq,
    O: OccupiedEntry<T>,
{
    #[inline]
    fn drop(&mut self) {
        let mut entry = self.0.take().unwrap_or_else(|| unreachable!());
        if entry.get_mut() == &Default::default() {
            entry.remove();
        }
    }
}

impl<T, V, O> Deref for DefaultEntry<T, V, O>
where
    T: Default + Eq,
    V: VacantEntry<T>,
    O: OccupiedEntry<T>,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            DefaultEntry::Vacant(ref entry) => entry,
            DefaultEntry::Occupied(ref entry) => entry,
        }
    }
}

impl<T, V, O> DerefMut for DefaultEntry<T, V, O>
where
    T: Default + Eq,
    V: VacantEntry<T>,
    O: OccupiedEntry<T>,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            DefaultEntry::Vacant(ref mut entry) => entry,
            DefaultEntry::Occupied(ref mut entry) => entry,
        }
    }
}

pub trait VacantEntry<T> {
    fn insert(self, value: T);
}

pub trait OccupiedEntry<T> {
    fn get(&self) -> &T;
    fn get_mut(&mut self) -> &mut T;
    fn remove(self);
}

impl<K, V> DefaultMap<K, V> for HashMap<K, V>
where
    K: Hash + Eq,
    V: Default + Eq,
{
    type VacantEntry<'a> = hash_map::VacantEntry<'a, K, V>
    where
        Self: 'a;

    type OccupiedEntry<'a> = hash_map::OccupiedEntry<'a, K, V>
    where
        Self: 'a;

    #[inline]
    fn entry_or_default(
        &mut self,
        key: K,
    ) -> DefaultEntry<V, Self::VacantEntry<'_>, Self::OccupiedEntry<'_>> {
        match self.entry(key) {
            hash_map::Entry::Occupied(entry) => DefaultOccupiedEntry::new(entry).into(),
            hash_map::Entry::Vacant(entry) => DefaultVacantEntry::new(entry).into(),
        }
    }
}

impl<'a, K, V> VacantEntry<V> for hash_map::VacantEntry<'a, K, V> {
    #[inline]
    fn insert(self, value: V) {
        self.insert(value);
    }
}

impl<'a, K, V> OccupiedEntry<V> for hash_map::OccupiedEntry<'a, K, V> {
    #[inline]
    fn get(&self) -> &V {
        self.get()
    }

    #[inline]
    fn get_mut(&mut self) -> &mut V {
        self.get_mut()
    }

    #[inline]
    fn remove(self) {
        self.remove();
    }
}

impl<K, V> DefaultMap<K, V> for BTreeMap<K, V>
where
    K: Ord,
    V: Default + Eq,
{
    type VacantEntry<'a> = btree_map::VacantEntry<'a, K, V>
    where
        Self: 'a;

    type OccupiedEntry<'a> = btree_map::OccupiedEntry<'a, K, V>
    where
        Self: 'a;

    #[inline]
    fn entry_or_default(
        &mut self,
        key: K,
    ) -> DefaultEntry<V, Self::VacantEntry<'_>, Self::OccupiedEntry<'_>> {
        match self.entry(key) {
            btree_map::Entry::Occupied(entry) => DefaultOccupiedEntry::new(entry).into(),
            btree_map::Entry::Vacant(entry) => DefaultVacantEntry::new(entry).into(),
        }
    }
}

impl<'a, K, V> VacantEntry<V> for btree_map::VacantEntry<'a, K, V>
where
    K: Ord,
{
    #[inline]
    fn insert(self, value: V) {
        self.insert(value);
    }
}

impl<'a, K, V> OccupiedEntry<V> for btree_map::OccupiedEntry<'a, K, V>
where
    K: Ord,
{
    #[inline]
    fn get(&self) -> &V {
        self.get()
    }

    #[inline]
    fn get_mut(&mut self) -> &mut V {
        self.get_mut()
    }

    #[inline]
    fn remove(self) {
        self.remove();
    }
}

impl<K, V, H> DefaultMap<K, V> for IterableMap<K, V, H>
where
    K: Ord + Clone + BorshSerialize + BorshDeserialize,
    V: Default + Eq + BorshSerialize + BorshDeserialize,
    H: ToKey,
{
    type VacantEntry<'a> = iterable_map::VacantEntry<'a, K, V, H>
    where
        Self: 'a;

    type OccupiedEntry<'a> = iterable_map::OccupiedEntry<'a, K, V, H>
    where
        Self: 'a;

    #[inline]
    fn entry_or_default(
        &mut self,
        key: K,
    ) -> DefaultEntry<V, Self::VacantEntry<'_>, Self::OccupiedEntry<'_>> {
        match self.entry(key) {
            iterable_map::Entry::Occupied(entry) => DefaultOccupiedEntry::new(entry).into(),
            iterable_map::Entry::Vacant(entry) => DefaultVacantEntry::new(entry).into(),
        }
    }
}

impl<'a, K, V, H> VacantEntry<V> for iterable_map::VacantEntry<'a, K, V, H>
where
    K: Ord + Clone + BorshSerialize + BorshDeserialize,
    V: BorshSerialize + BorshDeserialize,
    H: ToKey,
{
    #[inline]
    fn insert(self, value: V) {
        self.insert(value);
    }
}

impl<'a, K, V, H> OccupiedEntry<V> for iterable_map::OccupiedEntry<'a, K, V, H>
where
    K: Ord + Clone + BorshSerialize + BorshDeserialize,
    V: BorshSerialize + BorshDeserialize,
    H: ToKey,
{
    #[inline]
    fn get(&self) -> &V {
        self.get()
    }

    #[inline]
    fn get_mut(&mut self) -> &mut V {
        self.get_mut()
    }

    #[inline]
    fn remove(self) {
        self.remove();
    }
}
