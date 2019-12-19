use crate::project;
use euclid::{Point2D, Point3D, Vector2D, Vector3D};

/// Trait for defining the associated element for a geometrical space.
pub trait Space: Sized {
    /// Type of the elements of the space.
    type T: Copy;
}

/// Convenience trait for projecting vectors into a target space.
pub trait ProjectVec<U: Space> {
    fn project<Dst: project::From<U>>(self) -> Vector2D<Dst::T, Dst>;
}

impl<U: Space> ProjectVec<U> for Vector2D<U::T, U> {
    fn project<Dst: project::From<U>>(self) -> Vector2D<Dst::T, Dst> { Dst::vec_from(self) }
}

pub trait ProjectPoint<U: Space> {
    fn project<Dst: project::From<U>>(self) -> Point2D<Dst::T, Dst>;
}

impl<U: Space> ProjectPoint<U> for Point2D<U::T, U> {
    fn project<Dst: project::From<U>>(self) -> Point2D<Dst::T, Dst> { Dst::point_from(self) }
}

pub trait ProjectVec32<U: Space> {
    fn project<Dst: project::From32<U>>(self) -> Vector2D<Dst::T, Dst>;
}

impl<U: Space> ProjectVec32<U> for Vector3D<U::T, U> {
    fn project<Dst: project::From32<U>>(self) -> Vector2D<Dst::T, Dst> { Dst::vec_from(self) }
}

pub trait ProjectPoint32<U: Space> {
    fn project<Dst: project::From32<U>>(self) -> Point2D<Dst::T, Dst>;
}

impl<U: Space> ProjectPoint32<U> for Point3D<U::T, U> {
    fn project<Dst: project::From32<U>>(self) -> Point2D<Dst::T, Dst> { Dst::point_from(self) }
}
