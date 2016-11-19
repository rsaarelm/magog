use std::collections::BTreeMap;
use location::Location;

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct Field<T: PartialEq> {
    pub default: T,
    patch: BTreeMap<Location, T>,
}

impl<T: Copy + PartialEq> Field<T> {
    pub fn new(default: T) -> Field<T> {
        Field {
            patch: BTreeMap::new(),
            default: default,
        }
    }

    pub fn get(&self, pos: Location) -> T {
        self.patch.get(&pos).map(|&x| x).unwrap_or(self.default)
    }

    pub fn set(&mut self, pos: Location, val: T) {
        if val == self.default {
            self.patch.remove(&pos);
        } else {
            self.patch.insert(pos, val);
        }
    }
}
