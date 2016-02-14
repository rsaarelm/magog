use std::cmp::PartialOrd;
use num::{Num, Signed, Zero, One, abs};
use {Anchor, Shape2D};

/// A rectangle type.
#[derive(Copy, Clone, PartialEq, PartialOrd, Hash, Default, Debug, Serialize, Deserialize)]
pub struct Rect<T: Copy> {
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

    /// Get a point at an anchor position in the rectangle.
    pub fn point(&self, point: Anchor) -> [T; 2] {
        let one: T = One::one();
        let two = one + one;
        match point {
            Anchor::TopLeft => self.top,
            Anchor::TopRight => [self.top[0] + self.size[0], self.top[1]],
            Anchor::BottomLeft => [self.top[0], self.top[1] + self.size[1]],
            Anchor::BottomRight => {
                [self.top[0] + self.size[0], self.top[1] + self.size[1]]
            }
            Anchor::Top => [self.top[0] + self.size[0] / two, self.top[1]],
            Anchor::Left => [self.top[0], self.top[1] + self.size[1] / two],
            Anchor::Right => {
                [self.top[0] + self.size[0], self.top[1] + self.size[1] / two]
            }
            Anchor::Bottom => {
                [self.top[0] + self.size[0] / two, self.top[1] + self.size[1]]
            }
            Anchor::Center => {
                [self.top[0] + self.size[0] / two,
                 self.top[1] + self.size[1] / two]
            }
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
    /// assert_eq!(x.tiles([1, 1]).map(|t| t.top).next(), Some([0, 0]));
    /// assert_eq!(x.tiles([1, 1]).count(), 8 * 16);
    ///
    /// let y = Rect::new([0.0, 0.0], [3.141, 2.718]);
    /// assert_eq!(y.tiles([1.0, 1.0]).count(), 3 * 2);
    ///
    /// let z = Rect::new([3, 6], [8, 16]);
    /// assert_eq!(z.tiles([1, 1]).map(|t| t.top).next(), Some([3, 6]));
    /// ```
    pub fn tiles<'a, V: Into<[T; 2]>>(&'a self, dim: V) -> RectIter<'a, T> {
        RectIter::new(self, dim.into())
    }

    /// Get the area of the rectangle.
    pub fn area(&self) -> T {
        self.size[0] * self.size[1]
    }

    /// Produce the smallest new rectangle that contains both input
    /// rectangles.
    pub fn merge(&self, other: &Rect<T>) -> Rect<T> {
        Rect::new([*min(&self.top[0], &other.top[0]),
                   *min(&self.top[1], &other.top[1])],
                  [*max(&(self.top[0] + self.size[0]),
                        &(other.top[0] + other.size[0])),
                   *max(&(self.top[1] + self.size[1]),
                        &(other.top[1] + other.size[1]))])

    }

    /// Return whether this rectangle completely contains another rectangle.
    pub fn contains_rect(&self, other: &Rect<T>) -> bool {
        let p2 = self.point(Anchor::BottomRight);
        let q2 = other.point(Anchor::BottomRight);
        self.contains(other.point(Anchor::TopLeft)) && q2[0] <= p2[0] &&
        q2[1] <= p2[1]
    }
}

impl<T: Copy + Num + PartialOrd> Shape2D<T> for Rect<T> {
    fn bounding_box(&self) -> Rect<T> {
        *self
    }

    fn contains<V: Into<[T; 2]>>(&self, p: V) -> bool {
        let p = p.into();
        p[0] >= self.top[0] && p[1] >= self.top[1] &&
        p[0] < self.top[0] + self.size[0] &&
        p[1] < self.top[1] + self.size[1]
    }
}

/// Iterator for packed left-to-right top-to-bottom subrectangles
pub struct RectIter<'a, T: 'a + Copy> {
    base: &'a Rect<T>,
    dim: [T; 2],
    x: T,
    y: T,
}

impl<'a, T: Num + PartialOrd + Copy + 'a> RectIter<'a, T> {
    fn new(base: &'a Rect<T>, dim: [T; 2]) -> RectIter<T> {
        assert!(dim[0] > Zero::zero() && dim[1] > Zero::zero());
        RectIter {
            base: base,
            dim: dim,
            x: Zero::zero(),
            y: Zero::zero(),
        }
    }
}

impl<'a, T: Num + PartialOrd + Copy> Iterator for RectIter<'a, T> {
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
            top: [self.base.top[0] + self.x * self.dim[0],
                  self.base.top[1] + self.y * self.dim[1]],
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

fn max<'a, T: PartialOrd>(a: &'a T, b: &'a T) -> &'a T {
    if a.gt(b) {
        a
    } else {
        b
    }
}
