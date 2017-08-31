use location::Location;
use std::collections::BTreeMap;
use std::iter::{Extend, FromIterator};

#[derive(Clone, Serialize, Deserialize)]
pub struct Field<T: PartialEq> {
    patch: BTreeMap<Location, T>,
}

impl<T: Copy + PartialEq + Default> Field<T> {
    pub fn new() -> Field<T> { Field { patch: BTreeMap::new() } }

    pub fn get(&self, loc: Location) -> T {
        self.patch.get(&loc).cloned().unwrap_or(Default::default())
    }

    pub fn set(&mut self, loc: Location, val: T) {
        if val == Default::default() {
            self.patch.remove(&loc);
        } else {
            self.patch.insert(loc, val);
        }
    }

    /// Iterate non-default cells.
    pub fn iter(&self) -> ::std::collections::btree_map::Iter<Location, T> { self.patch.iter() }
}

impl<T: Copy + PartialEq + Default> FromIterator<(Location, T)> for Field<T> {
    fn from_iter<I: IntoIterator<Item = (Location, T)>>(iter: I) -> Self {
        Field { patch: iter.into_iter().collect() }
    }
}

impl<T: Copy + PartialEq + Default> Extend<(Location, T)> for Field<T> {
    fn extend<I: IntoIterator<Item = (Location, T)>>(&mut self, iter: I) {
        for (loc, val) in iter.into_iter() {
            self.set(loc, val);
        }
    }
}
