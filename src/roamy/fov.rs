use std::hashmap::HashMap;
use cgmath::point::{Point, Point2};

use area::Area;
use area::{Location, DIRECTIONS};

pub struct Fov(HashMap<Point2<int>, Location>);

impl Fov {
    pub fn new() -> Fov { Fov(HashMap::new()) }

    pub fn add(&mut self, other: ~Fov) {
        let &Fov(ref mut h) = self;
        let ~Fov(o) = other;
        h.extend(&mut o.move_iter());
    }
}

pub enum Type {
    Unknown,
    Remembered,
    Visible,
}

pub fn fov(_a: &Area, center: &Location, _radius: uint) -> Fov {
    let Fov(mut h) = Fov::new();
    // Dummy fov, just cover the immediate surrounding tiles.
    // TODO: Proper fov
    h.insert(Point2::new(0, 0), *center);
    for &v in DIRECTIONS.iter() {
        h.insert(Point2::new(0, 0).add_v(&v), center + v);
    }
    Fov(h)
}
