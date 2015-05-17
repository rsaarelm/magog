use std::ops::{Add, Sub, Mul, Div, Neg};
use std::cmp::{Ordering};
use num::{Zero, One};
use num::traits::{Num};
use image::{Primitive, GenericImage, Pixel};
use {Anchor};

/// 2D geometric vector.
#[derive(Copy, Debug, PartialEq, PartialOrd, Clone, Default, Hash, RustcDecodable, RustcEncodable)]
pub struct V2<T>(pub T, pub T);

impl<T: Eq> Eq for V2<T> { }

impl<T: Add<U, Output=V>, U, V> Add<V2<U>> for V2<T> {
    type Output = V2<V>;
    fn add(self, rhs: V2<U>) -> V2<V> {
        V2(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl<T: Sub<U, Output=V>, U, V> Sub<V2<U>> for V2<T> {
    type Output = V2<V>;
    fn sub(self, rhs: V2<U>) -> V2<V> {
        V2(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl<T: Neg<Output=U>, U> Neg<> for V2<T> {
    type Output = V2<U>;
    fn neg(self) -> V2<U> {
        V2(-self.0, -self.1)
    }
}

impl<T: Mul<U, Output=V>, U: Copy, V> Mul<U> for V2<T> {
    type Output = V2<V>;
    fn mul(self, rhs: U) -> V2<V> {
        V2(self.0 * rhs, self.1 * rhs)
    }
}

impl<T: Div<U, Output=V>, U: Copy, V> Div<U> for V2<T> {
    type Output = V2<V>;
    fn div(self, rhs: U) -> V2<V> {
        V2(self.0 / rhs, self.1 / rhs)
    }
}

impl<T> V2<T> {
    pub fn to_array(self) -> [T; 2] {
        [self.0, self.1]
    }
}

impl<T> V2<T> {
    pub fn map<U, F: Fn(T) -> U>(self, f: F) -> V2<U> {
        V2(f(self.0), f(self.1))
    }
}

impl<T: Num> V2<T> {
    /// Componentwise multiplication.
    pub fn mul(self, rhs: V2<T>) -> V2<T> {
        V2(self.0 * rhs.0, self.1 * rhs.1)
    }

    /// Componentwise division.
    pub fn div(self, rhs: V2<T>) -> V2<T> {
        V2(self.0 / rhs.0, self.1 / rhs.1)
    }

    /// Dot product.
    pub fn dot(self, rhs: V2<T>) -> T {
        self.0 * rhs.0 + self.1 * rhs.1
    }
}

impl<T: Ord+Copy> Ord for V2<T> {
    fn cmp(&self, other: &V2<T>) -> Ordering {
        (self.0, self.1).cmp(&(other.0, other.1))
    }
}

/// 3D geometric vector
#[derive(Copy, Debug, PartialEq, PartialOrd, Clone, Default, Hash, RustcDecodable, RustcEncodable)]
pub struct V3<T>(pub T, pub T, pub T);

impl<T: Eq> Eq for V3<T> { }

impl<T: Add<U, Output=V>, U, V> Add<V3<U>> for V3<T> {
    type Output = V3<V>;
    fn add(self, rhs: V3<U>) -> V3<V> {
        V3(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl<T: Sub<U, Output=V>, U, V> Sub<V3<U>> for V3<T> {
    type Output = V3<V>;
    fn sub(self, rhs: V3<U>) -> V3<V> {
        V3(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl<T: Neg<Output=U>, U> Neg<> for V3<T> {
    type Output = V3<U>;
    fn neg(self) -> V3<U> {
        V3(-self.0, -self.1, -self.2)
    }
}

impl<T: Mul<U, Output=V>, U: Copy, V> Mul<U> for V3<T> {
    type Output = V3<V>;
    fn mul(self, rhs: U) -> V3<V> {
        V3(self.0 * rhs, self.1 * rhs, self.2 * rhs)
    }
}

impl<T: Div<U, Output=V>, U: Copy, V> Div<U> for V3<T> {
    type Output = V3<V>;
    fn div(self, rhs: U) -> V3<V> {
        V3(self.0 / rhs, self.1 / rhs, self.2 / rhs)
    }
}

impl<T> V3<T> {
    pub fn to_array(self) -> [T; 3] {
        [self.0, self.1, self.2]
    }
}

impl<T> V3<T> {
    pub fn map<U, F: Fn(T) -> U>(self, f: F) -> V3<U> {
        V3(f(self.0), f(self.1), f(self.2))
    }
}

impl<T: Num> V3<T> {
    /// Componentwise multiplication.
    pub fn mul(self, rhs: V3<T>) -> V3<T> {
        V3(self.0 * rhs.0, self.1 * rhs.1, self.2 * rhs.2)
    }

    /// Componentwise division.
    pub fn div(self, rhs: V3<T>) -> V3<T> {
        V3(self.0 / rhs.0, self.1 / rhs.1, self.2 / rhs.2)
    }

    /// Dot product.
    pub fn dot(self, rhs: V3<T>) -> T {
        self.0 * rhs.0 + self.1 * rhs.1 + self.2 * rhs.2
    }
}

impl<T: Ord+Copy> Ord for V3<T> {
    fn cmp(&self, other: &V3<T>) -> Ordering {
        (self.0, self.1, self.2).cmp(&(other.0, other.1, other.2))
    }
}

/// A rectangle type consisting of position and size vectors.
#[derive(Copy, Debug, PartialEq, PartialOrd, Clone, Default, Hash, RustcDecodable, RustcEncodable)]
pub struct Rect<T>(pub V2<T>, pub V2<T>);

impl<T: Eq> Eq for Rect<T> { }

impl<T: Num+Copy+PartialOrd> Rect<T> {
    pub fn area(&self) -> T {
        (self.1).0 * (self.1).1
    }

    pub fn mn(&self) -> V2<T> {
        self.0
    }
    pub fn mx(&self) -> V2<T> {
        self.0 + self.1
    }
    pub fn dim(&self) -> V2<T> {
        self.1
    }

    pub fn point(&self, anchor: Anchor) -> V2<T> {
        let one: T = One::one();
        let two = one + one;
        //let two: T = One::one() + One::one();
        match anchor {
            Anchor::TopLeft => self.mn(),
            Anchor::TopRight => V2((self.0).0 + (self.1).0, (self.0).1),
            Anchor::BottomLeft => V2((self.0).0, (self.0).1 + (self.1).1),
            Anchor::BottomRight => self.mx(),
            Anchor::Top => V2((self.0).0 + (self.1).0 / two, (self.0).1),
            Anchor::Left => V2((self.0).0, (self.0).1 + (self.1).1 / two),
            Anchor::Right => V2((self.0).0 + (self.1).0, (self.0).1 + (self.1).1 / two),
            Anchor::Bottom => V2((self.0).0 + (self.1).0 / two, (self.0).1 + (self.1).1),
            Anchor::Center => V2((self.0).0 + (self.1).0 / two, (self.0).1 + (self.1).1 / two)
        }
    }

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

    pub fn contains(&self, p: &V2<T>) -> bool {
        let (mn, mx) = (self.mn(), self.mx());
        p.0 >= mn.0 && p.1 >= mn.1 && p.0 < mx.0 && p.1 < mx.1
    }

    pub fn edge_contains(&self, p: &V2<T>) -> bool {
        let (mn, mx) = (self.mn(), self.mx());
        let one = One::one();
        p.0 == mn.0 || p.1 == mn.1 || p.0 == mx.0 - one || p.1 == mx.1 - one
    }

    /// Return an iterator for all the points in the rectangle.
    pub fn iter(&self) -> RectIter<T> {
        RectIter {
            x: (self.0).0,
            y: (self.0).1,
            x0: (self.0).0,
            x1: (self.0).0 + (self.1).0,
            y1: (self.0).1 + (self.1).1,
        }
    }
}

impl<T: Add<U, Output=T> + Clone, U> Add<V2<U>> for Rect<T> {
    type Output = Rect<T>;
    fn add(self, rhs: V2<U>) -> Rect<T> {
        Rect(self.0 + rhs, self.1.clone())
    }
}

impl<T: Sub<U, Output=T> + Clone, U> Sub<V2<U>> for Rect<T> {
    type Output = Rect<T>;
    fn sub(self, rhs: V2<U>) -> Rect<T> {
        Rect(self.0 - rhs, self.1.clone())
    }
}

pub trait IterTiles<T> {
    /// Return an iterator of subtiles within self
    fn tiles(&self, tile_dim: V2<T>) -> TileIter<T>;
}

impl<T: Num+PartialOrd+Copy> IterTiles<T> for Rect<T> {
    fn tiles(&self, tile_dim: V2<T>) -> TileIter<T> {
        TileIter::new(*self, tile_dim)
    }
}

impl<I, P, T> IterTiles<u32> for I
    where I: GenericImage<Pixel=P>,
          P: Pixel<Subpixel=T>,
          T: Primitive
{
    fn tiles(&self, tile_dim: V2<u32>) -> TileIter<u32> {
        let (w, h) = self.dimensions();
        TileIter::new(Rect(V2(0, 0), V2(w, h)), tile_dim)
    }
}

/// Iterator for the integer points within a rectangle.
pub struct RectIter<T> {
    x: T,
    y: T,
    x0: T,
    x1: T,
    y1: T,
}

impl<T: Num+PartialOrd+Copy> Iterator for RectIter<T> {
    type Item = V2<T>;

    fn next(&mut self) -> Option<V2<T>> {
        if self.y >= self.y1 { return None; }
        let ret = Some(V2(self.x, self.y));
        self.x = self.x + One::one();
        if self.x >= self.x1 {
            self.x = self.x0;
            self.y = self.y + One::one();
        }
        ret
    }
}

/// Iterator for packed left-to-right top-to-bottom subrectangles
pub struct TileIter<T> {
    base: Rect<T>,
    tile_w: T,
    tile_h: T,
    x: T,
    y: T,
}

impl<T: Num+PartialOrd+Copy> TileIter<T> {
    pub fn new(base: Rect<T>, dim: V2<T>) -> TileIter<T> {
        assert!(dim.0 > Zero::zero() && dim.1 > Zero::zero());
        TileIter {
            base: base,
            tile_w: dim.0,
            tile_h: dim.1,
            x: Zero::zero(),
            y: Zero::zero()
        }
    }
}

impl<T: Num+PartialOrd+Copy> Iterator for TileIter<T> {
    type Item = Rect<T>;

    fn next(&mut self) -> Option<Rect<T>> {
        if self.tile_w > self.base.dim().0 { return None; }

        if (self.x + One::one()) * self.tile_w > self.base.dim().0 {
            self.y = self.y + One::one();
            self.x = Zero::zero();
        }

        if (self.y + One::one()) * self.tile_h > self.base.dim().1 {
            return None;
        }

        let ret = Rect(V2(self.x * self.tile_w, self.y * self.tile_h),
                       V2(self.tile_w, self.tile_h));
        self.x = self.x + One::one();
        Some(ret)
    }
}
