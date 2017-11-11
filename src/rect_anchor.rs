use euclid::{Point2D, TypedPoint2D, TypedRect, TypedSize2D, TypedVector2D, vec2};
use euclid::num::One;
use std::ops::{Add, Sub, Mul, Div};

/// Point anchoring, for snapping the origin point of a rectangle to different corners.
///
/// # Examples
///
/// ```
/// # extern crate euclid;
/// # extern crate vitral;
/// # fn main() {
/// use euclid::{Rect, rect, point2, size2};
/// use vitral::RectAnchor;
///
/// let bounds = rect(10, 10, 90, 90);
/// assert_eq!(bounds.anchor(&point2(-1, -1)), bounds.origin);   // Top left
/// assert_eq!(bounds.anchor(&point2(0, 0)), point2(55, 55));    // Center
/// assert_eq!(bounds.anchor(&point2(1, 1)), point2(100, 100));  // Bottom right
///
/// // Create a subrectangle snapped to anchor.
/// let anchor = point2(0, -1);
/// let widget: Rect<i32> = RectAnchor::anchored(&anchor, bounds.anchor(&anchor), size2(10, 10));
/// assert_eq!(widget, rect(50, 10, 10, 10));
///
/// let anchor = point2(1, 1);
/// let widget: Rect<i32> = RectAnchor::anchored(&anchor, bounds.anchor(&anchor), size2(10, 10));
/// assert_eq!(widget, rect(90, 90, 10, 10));
/// # }
/// ```
pub trait RectAnchor {
    type T;
    type Unit;

    /// Build a new instance with `anchor_point` snapped to the inner `anchor` position.
    ///
    /// The anchor maps [-1, 1] into the rectangle span for both x and y axes.
    fn anchored(
        anchor: &Point2D<Self::T>,
        anchor_point: TypedPoint2D<Self::T, Self::Unit>,
        size: TypedSize2D<Self::T, Self::Unit>,
    ) -> Self;

    /// Return an anchor point from inside the rectangle.
    ///
    /// The anchor maps [-1, 1] into the rectangle span for both x and y axes.
    fn anchor(&self, anchor: &Point2D<Self::T>) -> TypedPoint2D<Self::T, Self::Unit>;
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

impl<T, U> RectAnchor for TypedRect<T, U>
where
    T: One
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + Div<Output = T>
        + Copy,
{
    type T = T;
    type Unit = U;

    fn anchored(
        anchor: &Point2D<Self::T>,
        anchor_point: TypedPoint2D<Self::T, Self::Unit>,
        size: TypedSize2D<Self::T, Self::Unit>,
    ) -> Self {
        TypedRect::new(anchor_point - transform(anchor, &size), size)
    }

    fn anchor(&self, anchor: &Point2D<Self::T>) -> TypedPoint2D<Self::T, Self::Unit> {
        self.origin + transform(anchor, &self.size)
    }
}
