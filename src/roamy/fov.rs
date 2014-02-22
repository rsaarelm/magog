use std::hashmap::HashSet;

use area::Area;
use area::{Location, DIRECTIONS};

pub struct Fov(HashSet<Location>);

impl Fov {
    pub fn new() -> Fov { Fov(HashSet::new()) }

    pub fn add(&mut self, other: ~Fov) {
        let &Fov(ref mut h) = self;
        let ~Fov(o) = other;
        h.extend(&mut o.move_iter());
    }

    pub fn contains(&self, loc: &Location) -> bool {
        let &Fov(ref h) = self;
        h.contains(loc)
    }
}

pub fn fov(_a: &Area, center: &Location, _radius: uint) -> Fov {
    let Fov(mut h) = Fov::new();
    // Dummy fov, just cover the immediate surrounding tiles.
    // TODO: Proper fov
    h.insert(*center);
    for &v in DIRECTIONS.iter() {
        h.insert(center + v);
    }
    Fov(h)
}
