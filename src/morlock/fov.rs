use std::num::{Round};
use num::Integer;
use collections::hashmap::HashSet;

use cgmath::vector::{Vector, Vec2};

use area::Area;
use area::{Location, DIRECTIONS6, DIRECTIONS8};

pub struct Fov(HashSet<Location>);

#[deriving(Eq)]
struct Angle {
    pos: f32,
    radius: uint
}

impl Angle {
    pub fn new(pos: f32, radius: uint) -> Angle { Angle { pos: pos, radius: radius } }
    pub fn winding_index(self) -> int { (self.pos + 0.5).floor() as int }
    pub fn end_index(self) -> int { (self.pos + 0.5).ceil() as int }
    pub fn is_below(self, other: Angle) -> bool { self.winding_index() < other.end_index() }
    pub fn to_vec(self) -> Vec2<int> {
        if self.radius == 0 {
            return Vec2::new(0, 0);
        }

        let index = self.winding_index();

        let sector = index.mod_floor(&(self.radius as int * 6)) / self.radius as int;
        let offset = index.mod_floor(&(self.radius as int)) as int;
        let rod = DIRECTIONS6[sector].mul_s(self.radius as int);
        let tangent = DIRECTIONS6[(sector + 2) % 6].mul_s(offset);
        rod.add_v(&tangent)
    }

    pub fn further(self) -> Angle {
        Angle::new(
            self.pos * (self.radius + 1) as f32 / self.radius as f32,
            self.radius + 1)
    }

    pub fn next(self) -> Angle {
        Angle::new((self.pos + 0.5).floor() + 0.5, self.radius)
    }
}

impl Fov {
    pub fn new() -> Fov { Fov(HashSet::new()) }

    pub fn add(&mut self, other: ~Fov) {
        let &Fov(ref mut h) = self;
        let ~Fov(o) = other;
        h.extend(&mut o.move_iter());
    }

    pub fn contains(&self, loc: Location) -> bool {
        let &Fov(ref h) = self;
        h.contains(&loc)
    }

    fn insert(&mut self, loc: Location) {
        let Fov(ref mut h) = *self;
        h.insert(loc);
    }
}

pub fn fov(a: &Area, center: Location, range: uint) -> Fov {
    let mut ret = Fov::new();
    // Dummy fov, just cover the immediate surrounding tiles.
    // TODO: Proper fov
    ret.insert(center);
    // Use dir8 to make walls look nice.

    // XXX: Should only show the degenerate directions (-1, 1) and (1, -1) if
    // there's a wall there.
    for &v in DIRECTIONS8.iter() {
        ret.insert(center + v);
    }

    process(a, &mut ret, range, center, Angle::new(0.0, 1), Angle::new(6.0, 1));

    fn process(
        a: &Area, f: &mut Fov, range: uint,
        center: Location, begin: Angle, end: Angle) {
        if begin.radius > range { return; }

        let mut angle = begin;
        let group_opaque = a.is_opaque(center + angle.to_vec());
        while angle.is_below(end) {
            let loc = center + angle.to_vec();
            if a.is_opaque(loc) != group_opaque {
                process(a, f, range, center, angle, end);
                // Terrain opaquity has changed, time to recurse.
                if !group_opaque {
                    process(a, f, range, center, begin.further(), angle.further());
                }
                return;
            }
            f.insert(loc);

            angle = angle.next();
        }

        if !group_opaque {
            process(a, f, range, center, begin.further(), end.further());
        }
    }

    // Post-processing hack to make acute corner wall tiles in fake-isometric
    // rooms visible.
    {
        let Fov(ref mut h) = ret;
        let mut queue = ~[];
        for &loc in h.iter() {
            //    above
            //  left right
            //     loc
            //
            // If both loc and above are visible, left and right will
            // be made visible if they are opaque.
            let above = loc + Vec2::new(-1, -1);
            let left = loc + Vec2::new(-1, 0);
            let right = loc + Vec2::new(0, -1);
            if h.contains(&above) {
               if a.is_opaque(left) {
                   queue.push(left);
               }
               if a.is_opaque(right) {
                   queue.push(right);
               }
            }
        }

        for &loc in queue.iter() { h.insert(loc); }
    }

    ret
}
