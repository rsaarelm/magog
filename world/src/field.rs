use location::Location;
use std::collections::BTreeMap;

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
    pub fn _iter(&self) -> ::std::collections::btree_map::Iter<Location, T> { self.patch.iter() }
}
