use std::fmt::{Debug, Error, Formatter};
use std::hash::Hash;
use std::iter::FromIterator;

#[derive(Hash, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub struct VecSet<T: Ord>(Vec<T>);

impl<T: Hash + Debug + Clone + Ord> Debug for VecSet<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.debug_set().entries(self.0.iter()).finish()
    }
}

impl<T: Ord> VecSet<T> {
    pub fn contains(&self, item: &T) -> bool {
        match self.0.binary_search(item) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn remove(&mut self, item: &T) -> bool {
        match self.0.binary_search(item) {
            Ok(index) => {
                self.0.remove(index);
                true
            }
            Err(_) => false,
        }
    }

    pub fn insert(&mut self, item: T) -> bool {
        match self.0.binary_search(&item) {
            Ok(_) => false,
            Err(index) => {
                self.0.insert(index, item);
                true
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item=&T> {
        self.0.iter()
    }
}

impl<T: Ord> Default for VecSet<T> {
    fn default() -> Self {
        VecSet(Vec::default())
    }
}

impl<T: Ord> Extend<T> for VecSet<T> {
    fn extend<I: IntoIterator<Item=T>>(&mut self, iter: I) {
        self.0.extend(iter);
        self.0.sort_unstable();
    }
}

impl<T: Ord> FromIterator<T> for VecSet<T> {
    fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Self {
        let mut vec = Vec::from_iter(iter);
        vec.sort_unstable();
        VecSet(vec)
    }
}

impl<T: Ord> IntoIterator for VecSet<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T: Ord> IntoIterator for &'a VecSet<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}