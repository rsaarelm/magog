use euclid::{Point2D, TypedPoint2D, TypedRect, TypedSize2D, TypedVector2D, size2, vec2};
use euclid::num::{One, Zero};
use std::ops::{Add, Div, Mul, Neg, Sub};

/// Point anchoring, for snapping the origin point of a rectangle to different corners.
///
/// # Examples
///
/// ```
/// # extern crate euclid;
/// # extern crate vitral;
/// # fn main() {
/// use euclid::{Rect, rect, point2, size2};
/// use vitral::RectUtil;
///
/// let bounds = rect(10, 10, 90, 90);
/// assert_eq!(bounds.anchor(&point2(-1, -1)), bounds.origin);   // Top left
/// assert_eq!(bounds.anchor(&point2(0, 0)), point2(55, 55));    // Center
/// assert_eq!(bounds.anchor(&point2(1, 1)), point2(100, 100));  // Bottom right
///
/// // Create a subrectangle snapped to anchor.
/// let anchor = point2(0, -1);
/// let widget: Rect<i32> = bounds.anchored(&anchor, size2(10, 10));
/// assert_eq!(widget, rect(50, 10, 10, 10));
///
/// let anchor = point2(1, 1);
/// let widget: Rect<i32> = bounds.anchored(&anchor, size2(10, 10));
/// assert_eq!(widget, rect(90, 90, 10, 10));
/// # }
/// ```
pub trait RectUtil: Sized {
    type T;
    type Unit;

    /// Build a new instance with `anchor_point` snapped to the inner `anchor` position.
    ///
    /// The anchor maps [-1, 1] into the rectangle span for both x and y axes.
    fn new_anchored(
        anchor: &Point2D<Self::T>,
        anchor_point: TypedPoint2D<Self::T, Self::Unit>,
        size: TypedSize2D<Self::T, Self::Unit>,
    ) -> Self;

    /// Build a new rectangle at the given anchor point in this one.
    fn anchored(&self, anchor: &Point2D<Self::T>, size: TypedSize2D<Self::T, Self::Unit>) -> Self;

    /// Return an anchor point from inside the rectangle.
    ///
    /// The anchor maps [-1, 1] into the rectangle span for both x and y axes.
    fn anchor(&self, anchor: &Point2D<Self::T>) -> TypedPoint2D<Self::T, Self::Unit>;

    /// Shrink `size` by (1, 1) for code that expects the bottom and right points to be inside.
    fn inclusivize(&self) -> Self;

    /// Return main and split-off half of the rectangle.
    ///
    /// A positive `at_y` will split off from the top of the rectangle, a negative one will be
    /// split off from the bottom. The first return value will be the part of the main rectangle
    /// remaining after the split, the second will be the split-off top or bottom part.
    fn horizontal_split(&self, at_y: Self::T) -> (Self, Self);

    /// Return main and split-off half of the rectangle.
    ///
    /// A positive `at_x` will split off from the left of the rectangle, a negative one will be
    /// split off from the right. The first return value will be the part of the main rectangle
    /// remaining after the split, the second will be the split-off left or right part.
    fn vertical_split(&self, at_x: Self::T) -> (Self, Self);
}

fn transform<T, U>(anchor: &Point2D<T>, size: &TypedSize2D<T, U>) -> TypedVector2D<T, U>
where
    T: One + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T> + Copy,
{
    let two = T::one() + T::one();
    vec2(
        size.width * (anchor.x + T::one()) / two,
        size.height * (anchor.y + T::one()) / two,
    )
}

impl<T, U> RectUtil for TypedRect<T, U>
where
    T: Zero
        + One
        + PartialOrd
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + Div<Output = T>
        + Neg<Output = T>
        + Copy,
{
    type T = T;
    type Unit = U;

    fn new_anchored(
        anchor: &Point2D<Self::T>,
        anchor_point: TypedPoint2D<Self::T, Self::Unit>,
        size: TypedSize2D<Self::T, Self::Unit>,
    ) -> Self {
        TypedRect::new(anchor_point - transform(anchor, &size), size)
    }

    fn anchored(&self, anchor: &Point2D<Self::T>, size: TypedSize2D<Self::T, Self::Unit>) -> Self {
        Self::new_anchored(anchor, self.anchor(anchor), size)
    }

    fn anchor(&self, anchor: &Point2D<Self::T>) -> TypedPoint2D<Self::T, Self::Unit> {
        self.origin + transform(anchor, &self.size)
    }

    fn inclusivize(&self) -> Self {
        TypedRect::new(self.origin, self.size - size2(One::one(), One::one()))
    }

    fn horizontal_split(&self, at_y: Self::T) -> (Self, Self) {
        if at_y < Zero::zero() {
            (
                TypedRect::new(self.origin, self.size + size2(Zero::zero(), at_y)),
                TypedRect::new(
                    self.origin + size2(Zero::zero(), self.size.height + at_y),
                    size2(self.size.width, -at_y),
                ),
            )
        } else {
            let offset = size2(Zero::zero(), at_y);
            (
                TypedRect::new(self.origin + offset, self.size - offset),
                TypedRect::new(self.origin, size2(self.size.width, at_y)),
            )
        }
    }

    fn vertical_split(&self, at_x: Self::T) -> (Self, Self) {
        if at_x < Zero::zero() {
            (
                TypedRect::new(self.origin, self.size + size2(at_x, Zero::zero())),
                TypedRect::new(
                    self.origin + size2(self.size.width + at_x, Zero::zero()),
                    size2(-at_x, self.size.height),
                ),
            )
        } else {
            let offset = size2(at_x, Zero::zero());
            (
                TypedRect::new(self.origin + offset, self.size - offset),
                TypedRect::new(self.origin, size2(at_x, self.size.height)),
            )
        }
    }
}
