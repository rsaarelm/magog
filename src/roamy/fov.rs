use collections::hashmap::HashSet;

use area::Area;
use area::{Location, DIRECTIONS8};

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
    // Use dir8 to make walls look nice.

    // XXX: Should only show the degenerate directions (-1, 1) and (1, -1) if
    // there's a wall there.
    for &v in DIRECTIONS8.iter() {
        h.insert(center + v);
    }
    Fov(h)
}
