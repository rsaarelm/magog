use location::Location;
use std::collections::BTreeMap;

#[derive(Clone, Serialize, Deserialize)]
pub struct Field<T: PartialEq> {
    pub default: T,
    patch: BTreeMap<Location, T>,
}

impl<T: Copy + PartialEq> Field<T> {
    pub fn new(default: T) -> Field<T> {
        Field {
            patch: BTreeMap::new(),
            default,
        }
    }

    pub fn get(&self, pos: Location) -> T { self.patch.get(&pos).cloned().unwrap_or(self.default) }

    pub fn overrides(&self, pos: Location) -> bool { self.patch.contains_key(&pos) }

    pub fn set(&mut self, pos: Location, val: T) {
        if val == self.default {
            self.patch.remove(&pos);
        } else {
            self.patch.insert(pos, val);
        }
    }
}
