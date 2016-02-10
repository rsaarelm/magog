#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate cgmath;
extern crate num;
extern crate serde;

mod projection;
mod rect;

pub use projection::Projection;
pub use rect::Rect;

use num::{Num, Signed};

/// Rectangle anchoring points.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Anchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Top,
    Left,
    Right,
    Bottom,
    Center,
}

/// Generic 2D shape properties.
///
/// A Vec of shapes is interpreted as a union of the member shapes.
pub trait Shape2D<T: Copy> {
    fn bounding_box(&self) -> Rect<T>;

    fn contains<V: Into<[T; 2]>>(&self, _p: V) -> bool {
        false
    }
}

impl<'a, T, S> Shape2D<T> for Vec<S>
    where T: Copy + PartialOrd + Num + Signed,
          S: Shape2D<T>
{
    fn bounding_box(&self) -> Rect<T> {
        let mut it = self.iter();
        // Will panic if union is empty.
        let mut bounds = it.next().unwrap().bounding_box();

        loop {
            if let Some(b) = it.next().map(|x| x.bounding_box()) {
                bounds = bounds.merge(&b);
            } else {
                return bounds;
            }
        }
    }

    fn contains<V: Into<[T; 2]>>(&self, p: V) -> bool {
        let p = p.into();
        for i in self.iter() {
            if i.contains(p) {
                return true;
            }
        }

        false
    }
}

impl<'a, T> Shape2D<T> for [T; 2]
    where T: Copy + Num + PartialOrd + Num + Signed
{
    fn bounding_box(&self) -> Rect<T> {
        Rect::new(*self, *self)
    }

    fn contains<V: Into<[T; 2]>>(&self, p: V) -> bool {
        p.into() == *self
    }
}
