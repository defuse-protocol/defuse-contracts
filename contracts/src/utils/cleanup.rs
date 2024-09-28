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

/// A mapping where non-existing keys considered to have [`Default`] values
pub trait DefaultMap {
    type Key;
    type Value: Default + Eq;

    type VacantEntry<'a>: VacantEntry<Self::Key, Self::Value>
    where
        Self: 'a;
    type OccupiedEntry<'a>: OccupiedEntry<Self::Key, Self::Value>
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
    /// let mut m = HashMap::new();
    /// *m.entry_or_default("a") += 1;
    /// assert_eq!(m.get("a"), Some(&1));
    /// *m.entry_or_default("a") -= 1;
    /// assert_eq!(m.get("a"), None);
    /// ```
    fn entry_or_default(
        &mut self,
        key: Self::Key,
    ) -> DefaultEntry<Self::Key, Self::Value, Self::VacantEntry<'_>, Self::OccupiedEntry<'_>>;
}

pub enum DefaultEntry<K, V, VE, OE>
where
    V: Default + Eq,
    VE: VacantEntry<K, V>,
    OE: OccupiedEntry<K, V>,
{
    Vacant(DefaultVacantEntry<K, V, VE>),
    Occupied(DefaultOccupiedEntry<K, V, OE>),
}

impl<K, V, VE, OE> DefaultEntry<K, V, VE, OE>
where
    V: Default + Eq,
    VE: VacantEntry<K, V>,
    OE: OccupiedEntry<K, V>,
{
    #[inline]
    pub fn key(&self) -> &K {
        match self {
            Self::Vacant(entry) => entry.key(),
            Self::Occupied(entry) => entry.key(),
        }
    }

    #[inline]
    pub fn remove(self) -> V {
        match self {
            Self::Vacant(entry) => entry.remove(),
            Self::Occupied(entry) => entry.remove(),
        }
    }
}

impl<K, V, VE, OE> Deref for DefaultEntry<K, V, VE, OE>
where
    V: Default + Eq,
    VE: VacantEntry<K, V>,
    OE: OccupiedEntry<K, V>,
{
    type Target = V;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            DefaultEntry::Vacant(ref entry) => entry,
            DefaultEntry::Occupied(ref entry) => entry,
        }
    }
}

impl<K, V, VE, OE> DerefMut for DefaultEntry<K, V, VE, OE>
where
    V: Default + Eq,
    VE: VacantEntry<K, V>,
    OE: OccupiedEntry<K, V>,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            DefaultEntry::Vacant(ref mut entry) => entry,
            DefaultEntry::Occupied(ref mut entry) => entry,
        }
    }
}

impl<K, V, VE, OE> From<DefaultVacantEntry<K, V, VE>> for DefaultEntry<K, V, VE, OE>
where
    V: Default + Eq,
    VE: VacantEntry<K, V>,
    OE: OccupiedEntry<K, V>,
{
    #[inline]
    fn from(entry: DefaultVacantEntry<K, V, VE>) -> Self {
        Self::Vacant(entry)
    }
}

impl<K, V, VE, OE> From<DefaultOccupiedEntry<K, V, OE>> for DefaultEntry<K, V, VE, OE>
where
    V: Default + Eq,
    VE: VacantEntry<K, V>,
    OE: OccupiedEntry<K, V>,
{
    #[inline]
    fn from(entry: DefaultOccupiedEntry<K, V, OE>) -> Self {
        Self::Occupied(entry)
    }
}

pub struct DefaultVacantEntry<K, V, E>(Option<(V, E)>, PhantomData<K>)
where
    V: Default + Eq,
    E: VacantEntry<K, V>;

impl<K, V, E> DefaultVacantEntry<K, V, E>
where
    V: Default + Eq,
    E: VacantEntry<K, V>,
{
    #[inline]
    pub fn new(entry: E) -> Self {
        Self(Some((V::default(), entry)), PhantomData)
    }

    #[inline]
    pub fn key(&self) -> &K {
        self.0.as_ref().unwrap_or_else(|| unreachable!()).1.key()
    }

    #[inline]
    pub fn remove(mut self) -> V {
        self.0.take().unwrap_or_else(|| unreachable!()).0
    }
}

impl<K, V, E> Deref for DefaultVacantEntry<K, V, E>
where
    V: Default + Eq,
    E: VacantEntry<K, V>,
{
    type Target = V;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0.as_ref().unwrap_or_else(|| unreachable!()).0
    }
}

impl<K, V, E> DerefMut for DefaultVacantEntry<K, V, E>
where
    V: Default + Eq,
    E: VacantEntry<K, V>,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.as_mut().unwrap_or_else(|| unreachable!()).0
    }
}

impl<K, V, E> Drop for DefaultVacantEntry<K, V, E>
where
    V: Default + Eq,
    E: VacantEntry<K, V>,
{
    #[inline]
    fn drop(&mut self) {
        let Some((v, entry)) = self.0.take() else {
            return;
        };
        if v != Default::default() {
            entry.insert(v);
        }
    }
}

pub struct DefaultOccupiedEntry<K, T, O>(Option<O>, PhantomData<(K, T)>)
where
    T: Default + Eq,
    O: OccupiedEntry<K, T>;

impl<K, V, E> DefaultOccupiedEntry<K, V, E>
where
    V: Default + Eq,
    E: OccupiedEntry<K, V>,
{
    #[inline]
    pub fn new(entry: E) -> Self {
        Self(Some(entry), PhantomData)
    }

    #[inline]
    pub fn key(&self) -> &K {
        self.0.as_ref().unwrap_or_else(|| unreachable!()).key()
    }

    #[inline]
    pub fn remove(mut self) -> V {
        self.0.take().unwrap_or_else(|| unreachable!()).remove()
    }
}

impl<K, V, E> Deref for DefaultOccupiedEntry<K, V, E>
where
    V: Default + Eq,
    E: OccupiedEntry<K, V>,
{
    type Target = V;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap_or_else(|| unreachable!()).get()
    }
}

impl<K, V, E> DerefMut for DefaultOccupiedEntry<K, V, E>
where
    V: Default + Eq,
    E: OccupiedEntry<K, V>,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap_or_else(|| unreachable!()).get_mut()
    }
}

impl<K, V, E> Drop for DefaultOccupiedEntry<K, V, E>
where
    V: Default + Eq,
    E: OccupiedEntry<K, V>,
{
    #[inline]
    fn drop(&mut self) {
        let Some(entry) = self.0.take() else {
            return;
        };
        if entry.get() == &Default::default() {
            entry.remove();
        }
    }
}

pub trait VacantEntry<K, V> {
    fn key(&self) -> &K;
    fn into_key(self) -> K;
    fn insert(self, value: V);
}

pub trait OccupiedEntry<K, V> {
    fn key(&self) -> &K;
    fn get(&self) -> &V;
    fn get_mut(&mut self) -> &mut V;
    fn remove(self) -> V;
}

impl<K, V> DefaultMap for HashMap<K, V>
where
    K: Hash + Eq,
    V: Default + Eq,
{
    type Key = K;
    type Value = V;
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
    ) -> DefaultEntry<K, V, Self::VacantEntry<'_>, Self::OccupiedEntry<'_>> {
        match self.entry(key) {
            hash_map::Entry::Occupied(entry) => DefaultOccupiedEntry::new(entry).into(),
            hash_map::Entry::Vacant(entry) => DefaultVacantEntry::new(entry).into(),
        }
    }
}

impl<'a, K, V> VacantEntry<K, V> for hash_map::VacantEntry<'a, K, V> {
    #[inline]
    fn key(&self) -> &K {
        self.key()
    }

    #[inline]
    fn into_key(self) -> K {
        self.into_key()
    }

    #[inline]
    fn insert(self, value: V) {
        self.insert(value);
    }
}

impl<'a, K, V> OccupiedEntry<K, V> for hash_map::OccupiedEntry<'a, K, V> {
    #[inline]
    fn key(&self) -> &K {
        self.key()
    }

    #[inline]
    fn get(&self) -> &V {
        self.get()
    }

    #[inline]
    fn get_mut(&mut self) -> &mut V {
        self.get_mut()
    }

    #[inline]
    fn remove(self) -> V {
        self.remove()
    }
}

impl<K, V> DefaultMap for BTreeMap<K, V>
where
    K: Ord,
    V: Default + Eq,
{
    type Key = K;
    type Value = V;

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
    ) -> DefaultEntry<K, V, Self::VacantEntry<'_>, Self::OccupiedEntry<'_>> {
        match self.entry(key) {
            btree_map::Entry::Occupied(entry) => DefaultOccupiedEntry::new(entry).into(),
            btree_map::Entry::Vacant(entry) => DefaultVacantEntry::new(entry).into(),
        }
    }
}

impl<'a, K, V> VacantEntry<K, V> for btree_map::VacantEntry<'a, K, V>
where
    K: Ord,
{
    #[inline]
    fn key(&self) -> &K {
        self.key()
    }

    #[inline]
    fn into_key(self) -> K {
        self.into_key()
    }

    #[inline]
    fn insert(self, value: V) {
        self.insert(value);
    }
}

impl<'a, K, V> OccupiedEntry<K, V> for btree_map::OccupiedEntry<'a, K, V>
where
    K: Ord,
{
    #[inline]
    fn key(&self) -> &K {
        self.key()
    }

    #[inline]
    fn get(&self) -> &V {
        self.get()
    }

    #[inline]
    fn get_mut(&mut self) -> &mut V {
        self.get_mut()
    }

    #[inline]
    fn remove(self) -> V {
        self.remove()
    }
}

impl<K, V, H> DefaultMap for IterableMap<K, V, H>
where
    K: Ord + Clone + BorshSerialize + BorshDeserialize,
    V: Default + Eq + BorshSerialize + BorshDeserialize,
    H: ToKey,
{
    type Key = K;
    type Value = V;

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
    ) -> DefaultEntry<K, V, Self::VacantEntry<'_>, Self::OccupiedEntry<'_>> {
        match self.entry(key) {
            iterable_map::Entry::Occupied(entry) => DefaultOccupiedEntry::new(entry).into(),
            iterable_map::Entry::Vacant(entry) => DefaultVacantEntry::new(entry).into(),
        }
    }
}

impl<'a, K, V, H> VacantEntry<K, V> for iterable_map::VacantEntry<'a, K, V, H>
where
    K: Ord + Clone + BorshSerialize + BorshDeserialize,
    V: BorshSerialize + BorshDeserialize,
    H: ToKey,
{
    #[inline]
    fn key(&self) -> &K {
        self.key()
    }

    #[inline]
    fn into_key(self) -> K {
        self.into_key()
    }

    #[inline]
    fn insert(self, value: V) {
        self.insert(value);
    }
}

impl<'a, K, V, H> OccupiedEntry<K, V> for iterable_map::OccupiedEntry<'a, K, V, H>
where
    K: Ord + Clone + BorshSerialize + BorshDeserialize,
    V: BorshSerialize + BorshDeserialize,
    H: ToKey,
{
    #[inline]
    fn key(&self) -> &K {
        self.key()
    }

    #[inline]
    fn get(&self) -> &V {
        self.get()
    }

    #[inline]
    fn get_mut(&mut self) -> &mut V {
        self.get_mut()
    }

    #[inline]
    fn remove(self) -> V {
        self.remove()
    }
}
