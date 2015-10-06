use std::hash::{Hash};
use std::marker::{PhantomData};
use std::collections::{BTreeMap, HashMap};
use std::default::Default;

/// A backdrop maps all points of a field to some value.
///
/// It is usually a simple constant function.
pub trait Backdrop<D, R> {
    fn get(&self, pos: D) -> R;
}

#[derive(Copy, Clone, RustcEncodable, RustcDecodable)]
/// A backdrop that returns a single value for every field point.
pub struct ConstBackdrop<R>(pub R);

impl<D, R: Copy> Backdrop<D, R> for ConstBackdrop<R> {
    fn get(&self, _key: D) -> R { self.0 }
}

/// A patch dynamically overwrites some values in a field.
pub trait Patch<D, R>: Default {
    fn get<'a>(&'a self, pos: D) -> Option<&'a R>;
    fn set(&mut self, pos: D, val: R);
    fn clear(&mut self, pos: D);
}

impl<D: Ord, R> Patch<D, R> for BTreeMap<D, R> {
    fn get<'a>(&'a self, pos: D) -> Option<&'a R> { self.get(&pos) }
    fn set(&mut self, pos: D, val: R) { self.insert(pos, val); }
    fn clear(&mut self, pos: D) { self.remove(&pos); }
}

impl<D: Eq+Hash, R> Patch<D, R> for HashMap<D, R> {
    fn get<'a>(&'a self, pos: D) -> Option<&'a R> { self.get(&pos) }
    fn set(&mut self, pos: D, val: R) { self.insert(pos, val); }
    fn clear(&mut self, pos: D) { self.remove(&pos); }
}

#[derive(Clone, RustcEncodable, RustcDecodable)]
/// A generic field structure that overlays a sparse, mutable patch of values
/// on top of a backdrop type that provides default values for the entire
/// query domain.
pub struct Field<D, R, B, P> {
    backdrop: B,
    patch: P,
    _d: PhantomData<D>,
    _r: PhantomData<R>,
}

impl<D: Copy, R: Copy, B, P> Field<D, R, B, P>
    where R: Copy,
          B: Backdrop<D, R>,
          P: Patch<D, R>
{
    pub fn new(b: B) -> Field<D, R, B, P> {
        Field {
            backdrop: b,
            patch: Default::default(),
            _d: PhantomData,
            _r: PhantomData,
        }
    }

    /// Get the value of the field in the given point.
    ///
    /// If the value has been overriden by calling set, that value will be
    /// returned. Otherwise a value from the backdrop structure will be
    /// returned.
    pub fn get(&self, pos: D) -> R {
        match self.patch.get(pos) {
            Some(&v) => v,
            None => self.backdrop.get(pos)
        }
    }

    /// Override the value in a given point.
    pub fn set(&mut self, pos: D, val: R) { self.patch.set(pos, val); }

    /// Clear the value in a given point.
    pub fn clear(&mut self, pos: D) { self.patch.clear(pos); }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_field() {
        use std::collections::BTreeMap;
        use ::{Field, ConstBackdrop, Patch};
        use rustc_serialize::json;

        type MyField = Field<(i32, i32), i32, ConstBackdrop<i32>, BTreeMap<(i32, i32), i32>>;
        let mut field: MyField = Field::new(ConstBackdrop(-7));

        // Test basic ops.
        assert!(field.get((0, 0)) == -7);
        field.set((0, 0), 12);
        field.set((1, 2), 19);
        assert!(field.get((0, 0)) == 12);
        assert!(field.get((1, 2)) == 19);
        field.clear((0, 0));
        assert!(field.get((0, 0)) == -7);

        // Check that serialization works.
        let save = json::encode(&field).expect("Field JSON encoding failed");
        let field2 = json::decode::<MyField>(&save).expect("Field JSON decoding failed");
        assert!(field2.get((0, 0)) == -7);
        assert!(field2.get((1, 2)) == 19);
    }
}
