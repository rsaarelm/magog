/// 2D geometric vector.
#[deriving(Show, PartialEq, PartialOrd, Clone, Decodable, Encodable)]
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
#[deriving(Show, PartialEq, PartialOrd, Clone, Decodable, Encodable)]
pub struct Rect<T>(pub V2<T>, pub V2<T>);

impl<T: Primitive> Rect<T> {
    pub fn area(&self) -> T { (self.1).0 * (self.1).1 }

    pub fn mn(&self) -> V2<T> { self.0 }

    pub fn mx(&self) -> V2<T> { self.0 + self.1 }

    /// Grow the rectangle to enclose point p.
    pub fn grow(&mut self, p: V2<T>) {
        let (mn, mx) = (self.mn(), self.mx());

        if p.0 < mn.0 {
            (self.1).0 = (self.1).0 + mn.0 - p.0;
            (self.0).0 = p.0;
        }

        if p.1 < mn.1 {
            (self.1).1 = (self.1).1 + mn.1 - p.1;
            (self.0).1 = p.1;
        }

        if p.0 > mx.0 { (self.1).0 = p.0 - mn.0; }

        if p.1 > mx.1 { (self.1).1 = p.1 - mn.1; }
    }

    pub fn intersects(&self, rhs: &Rect<T>) -> bool {
        let (mn, mx) = (self.mn(), self.mx());
        let (rmn, rmx) = (rhs.mn(), rhs.mx());

        !(mx.0 <= rmn.0 || mn.0 >= rmx.0 ||
          mx.1 <= rmn.1 || mn.1 >= rmx.1)
    }
}
