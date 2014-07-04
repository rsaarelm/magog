use cgmath::point::{Point2};
use cgmath::aabb::{Aabb, Aabb2};
use std::num::{one};
use cgmath::num::BaseNum;
use rand::{Rng};
use rand::distributions::range::SampleRange;

pub trait RectUtil<S: BaseNum + SampleRange, I: Iterator<Point2<S>>> {
    // Iterate all integer points inside the rectangle.
    fn points(&self) -> I;
    // Get the scanline position (0 at top left corner, increasing along
    // positive x-axis) for a point inside the rectangle.
    fn scan_pos(&self, pos: &Point2<S>) -> int;
    // Convenience constructor with naked coordinates.
    fn new(x1: S, y1: S, x2: S, y2: S) -> Self;

    fn random<R: Rng>(&self, rng: &mut R) -> Point2<S>;
}

pub struct RectIter<S> {
    x: S,
    y: S,
    x_start: S,
    x_end: S,
    y_end: S,
}

impl<S: BaseNum> Iterator<Point2<S>> for RectIter<S> {
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

impl<S: BaseNum + SampleRange> RectUtil<S, RectIter<S>> for Aabb2<S> {
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

    fn random<R: Rng>(&self, rng: &mut R) -> Point2<S> {
        Point2::new(
            rng.gen_range(self.min.x.clone(), self.max.x.clone()),
            rng.gen_range(self.min.y.clone(), self.max.y.clone()))
    }
}
