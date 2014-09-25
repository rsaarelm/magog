/// 2D geometric vector.
#[deriving(Show, PartialEq, PartialOrd, Clone)]
pub struct V2<T>(pub T, pub T);

impl<T: Add<U, V>, U, V> Add<V2<U>, V2<V>> for V2<T> {
    fn add(&self, rhs: &V2<U>) -> V2<V> { V2(self.0 + rhs.0, self.1 + rhs.1) }
}

impl<T: Sub<U, V>, U, V> Sub<V2<U>, V2<V>> for V2<T> {
    fn sub(&self, rhs: &V2<U>) -> V2<V> { V2(self.0 - rhs.0, self.1 - rhs.1) }
}

impl<T: Neg<U>, U> Neg<V2<U>> for V2<T> {
    fn neg(&self) -> V2<U> { V2(-self.0, -self.1) }
}

impl<T> V2<T> {
    pub fn to_array(self) -> [T, ..2] { [self.0, self.1] }
}

impl<T> V2<T> {
    pub fn map<U>(self, f: |T| -> U) -> V2<U> { V2(f(self.0), f(self.1)) }
}

impl<T: Primitive> V2<T> {
    // Componentwise multiplication.
    pub fn mul(self, rhs: V2<T>) -> V2<T> { V2(self.0 * rhs.0, self.1 * rhs.1) }
}

/// A rectangle type consisting of position and size vectors.
#[deriving(Show, PartialEq, PartialOrd, Clone)]
pub struct Rect<T>(pub V2<T>, pub V2<T>);

impl<T: Primitive> Rect<T> {
    pub fn area(&self) -> T { (self.1).0 * (self.1).1 }

    pub fn min(&self) -> V2<T> { self.0 }

    pub fn max(&self) -> V2<T> { self.0 + self.1 }
}
