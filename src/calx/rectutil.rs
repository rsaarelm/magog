use cgmath::point::{Point2};
use cgmath::aabb::{Aabb, Aabb2};
use std::num::{one};

pub trait RectUtil<S: Primitive, I: Iterator<Point2<S>>> {
    // Iterate all integer points inside the rectangle.
    fn points(&self) -> I;
    // Get the scanline position (0 at top left corner, increasing along
    // positive x-axis) for a point inside the rectangle.
    fn scan_pos(&self, pos: &Point2<S>) -> int;
    // Convenience constructor with naked coordinates.
    fn new(x1: S, y1: S, x2: S, y2: S) -> Self;
}

pub struct RectIter<S> {
    priv x: S,
    priv y: S,
    priv x_start: S,
    priv x_end: S,
    priv y_end: S,
}

impl<S: Primitive> Iterator<Point2<S>> for RectIter<S> {
    #[inline]
    fn next(&mut self) -> Option<Point2<S>> {
        if self.x_end <= self.x_start {
            return None
        }
        if self.y >= self.y_end {
            return None
        }
        let ret = Point2::new(self.x.clone(), self.y.clone());
        self.x = self.x + one::<S>();
        if self.x == self.x_end {
            self.y = self.y + one::<S>();
            self.x = self.x_start.clone();
        }
        Some(ret)
    }

}

impl<S: Primitive> RectUtil<S, RectIter<S>> for Aabb2<S> {
    #[inline]
    fn points(&self) -> RectIter<S> {
        RectIter {
            x: self.min().x.clone(),
            y: self.min().y.clone(),
            x_start: self.min().x.clone(),
            x_end: self.max().x.clone(),
            y_end: self.max().y.clone(),
        }
    }

    #[inline]
    fn scan_pos(&self, pos: &Point2<S>) -> int {
        let delta_x = pos.x - self.min().x;
        let delta_y = pos.y - self.min().y;
        let pitch = self.max().x - self.min().x;
        (delta_x + pitch * delta_y).to_int().unwrap()
    }

    fn new(x1: S, y1: S, x2: S, y2: S) -> Aabb2<S> {
        Aabb2::new(Point2::new(x1, y1), Point2::new(x2, y2))
    }
}
