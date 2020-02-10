use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::{Debug, Error, Formatter};
use std::hash::Hash;

use itertools::Format;

#[derive(Clone, Hash)]
//TODO try hashmap again
pub struct UndoMap<K: Eq + Hash, V>(BTreeMap<K, V>);

impl<K: Eq + Hash + Ord, V> Default for UndoMap<K, V> {
    fn default() -> Self {
        UndoMap(Default::default())
    }
}

impl<K: Eq + Hash + Debug + Ord, V: Debug> Debug for UndoMap<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<K: Eq + Hash + Ord, V> UndoMap<K, V> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn keys(&self) -> std::collections::btree_map::Keys<'_, K, V> {
        self.0.keys()
    }

    pub fn iter(&self) -> std::collections::btree_map::Iter<K, V> {
        self.into_iter()
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.0.contains_key(key)
    }

    pub fn as_map(&self) -> &BTreeMap<K, V> {
        &self.0
    }
}

impl<'a, K: Eq + Hash, V> IntoIterator for &'a UndoMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = std::collections::btree_map::Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<K: Eq + Hash + Copy + Ord, V: Eq + PartialEq + Copy> UndoMap<K, V> {
    fn set_map_to(&mut self, key: K, value: Option<V>) -> Option<V> {
        match value {
            Some(value) => self.0.insert(key, value),
            None => self.0.remove(&key)
        }
    }

    fn set_to<R, F: FnMut(&mut Self) -> R>(&mut self, key: K, value: Option<V>, mut f: F) -> Option<R> {
        let prev = self.set_map_to(key, value);
        //nothing changed
        if prev == value { return None; }

        let len_before = self.len();
        let ret = f(self);
        debug_assert!(self.len() == len_before);

        let curr = self.set_map_to(key, prev);
        debug_assert!(curr == value, "map was changed during function call");

        Some(ret)
    }

    pub fn insert<R, F: FnMut(&mut Self) -> R>(&mut self, key: K, value: V, f: F) -> Option<R> {
        self.set_to(key, Some(value), f)
    }

    pub fn remove<R, F: FnMut(&mut Self) -> R>(&mut self, key: K, f: F) -> Option<R> {
        self.set_to(key, None, f)
    }
}