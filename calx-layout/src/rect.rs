use std::cmp::PartialOrd;
use num::{Num, Signed, Zero, One, abs};
use Anchor;

pub struct Rect<T> {
    /// Top left (closest to the origin) corner of the rectange.
    pub top: [T; 2],
    /// Size of the rectangle, all elements are assumed to always be positive.
    pub size: [T; 2],
}

impl<T> Rect<T> where T: Num + PartialOrd + Signed + Copy
{
    /// Create a new rectangle from two corner points.
    pub fn new<V: Into<[T; 2]>>(p1: V, p2: V) -> Rect<T> {
        let (p1, p2): ([T; 2], [T; 2]) = (p1.into(), p2.into());

        Rect {
            top: [*min(&p1[0], &p2[0]), *min(&p1[1], &p2[1])],
            size: [abs(p2[0] - p1[0]), abs(p2[1] - p1[1])],
        }
    }

    pub fn point(&self, point: Anchor) -> [T; 2] {
        let one: T = One::one();
        let two = one + one;
        match point {
            Anchor::TopLeft => self.top,
            Anchor::TopRight => [self.top[0] + self.size[0], self.top[1]],
            Anchor::BottomLeft => [self.top[0], self.top[1] + self.size[1]],
            Anchor::BottomRight => [self.top[0] + self.size[0], self.top[1] + self.size[1]],
            Anchor::Top => [self.top[0] + self.size[0] / two, self.top[1]],
            Anchor::Left => [self.top[0], self.top[1] + self.size[1] / two],
            Anchor::Right => [self.top[0] + self.size[0], self.top[1] + self.size[1] / two],
            Anchor::Bottom => [self.top[0] + self.size[0] / two, self.top[1] + self.size[1]],
            Anchor::Center => [self.top[0] + self.size[0] / two, self.top[1] + self.size[1] / two],
        }
    }

    /// Iterate tiles of the given size inside the rectangle.
    ///
    /// The tiles are ordered from left to right, then from top to bottom. The
    /// first tile snaps to the top left corner of the rectangle, and parts
    /// along the right and bottom edges of the rectangle are left uncovered
    /// if a whole tile will not fit in them.
    ///
    /// ```
    /// use calx_layout::Rect;
    ///
    /// let x = Rect::new([0, 0], [8, 16]);
    /// // To iterate integer points in the rectangle, just use [1, 1] tiles.
    /// assert_eq!(x.tiles([1, 1]).map(|x| x.top).next(), Some([0, 0]));
    /// assert_eq!(x.tiles([1, 1]).count(), 8 * 16);
    ///
    /// let y = Rect::new([0.0, 0.0], [3.141, 2.718]);
    /// assert_eq!(y.tiles([1.0, 1.0]).count(), 3 * 2);
    /// ```
    pub fn tiles<'a, V: Into<[T; 2]>>(&'a self, dim: V) -> TileIter<'a, T> {
        TileIter::new(self, dim.into())
    }
}

/// Iterator for packed left-to-right top-to-bottom subrectangles
pub struct TileIter<'a, T: 'a> {
    base: &'a Rect<T>,
    dim: [T; 2],
    x: T,
    y: T,
}

impl<'a, T: Num + PartialOrd + Copy + 'a> TileIter<'a, T> {
    fn new(base: &'a Rect<T>, dim: [T; 2]) -> TileIter<T> {
        assert!(dim[0] > Zero::zero() && dim[1] > Zero::zero());
        TileIter {
            base: base,
            dim: dim,
            x: Zero::zero(),
            y: Zero::zero(),
        }
    }
}

impl<'a, T: Num + PartialOrd + Copy> Iterator for TileIter<'a, T> {
    type Item = Rect<T>;

    fn next(&mut self) -> Option<Rect<T>> {
        if self.dim[0] > self.base.size[0] {
            return None;
        }

        if (self.x + One::one()) * self.dim[0] > self.base.size[0] {
            self.y = self.y + One::one();
            self.x = Zero::zero();
        }

        if (self.y + One::one()) * self.dim[1] > self.base.size[1] {
            return None;
        }

        let ret = Rect {
            top: [self.x * self.dim[0], self.y * self.dim[1]],
            size: self.dim,
        };
        self.x = self.x + One::one();
        Some(ret)
    }
}

fn min<'a, T: PartialOrd>(a: &'a T, b: &'a T) -> &'a T {
    if a.lt(b) {
        a
    } else {
        b
    }
}
