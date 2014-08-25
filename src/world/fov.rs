use num::Integer;
use std::collections::hashmap::HashSet;
use cgmath::vector::{Vector, Vector2};
use world::spatial::{Location, DIRECTIONS6};
use world::system::{World};
use world::area::Area;

#[deriving(Eq, PartialEq, Show)]
pub enum FovStatus {
    Seen,
    Remembered,
}

#[deriving(Clone)]
/// Field of view computation result.
pub struct Fov {
    seen: HashSet<Location>,
    remembered: HashSet<Location>,
}

impl Fov {
    pub fn new() -> Fov {
        Fov {
            seen: HashSet::new(),
            remembered: HashSet::new(),
        }
    }

    pub fn get(&self, loc: Location) -> Option<FovStatus> {
        if self.seen.contains(&loc) {
            Some(Seen)
        } else if self.remembered.contains(&loc) {
            Some(Remembered)
        } else {
            None
        }
    }

    pub fn update(&mut self, world: &World, center: Location, range: uint) {
        self.seen = HashSet::new();

        mark_seen(self, center);

        process(self, world, range, center, Angle::new(0.0, 1), Angle::new(6.0, 1));

        // Post-processing hack to make acute corner wall tiles in
        // fake-isometric rooms visible.
        {
            let mut queue = vec!();
            for &loc in self.seen.iter() {
                //    above
                //  left right
                //     loc
                //
                // If both loc and above are visible, left and right will
                // be made visible if they are opaque.
                //
                // (Loc is known to be seen from the fact that the iteration
                // brings us to this block to begin with.)
                let above = loc + Vector2::new(-1, -1);
                let left_loc = loc + Vector2::new(-1, 0);
                let right_loc = loc + Vector2::new(0, -1);

                if self.seen.contains(&above) {
                    if world.is_opaque(left_loc) {
                        queue.push(left_loc);
                    }
                    if world.is_opaque(right_loc) {
                        queue.push(right_loc);
                    }
                }
            }

            for &loc in queue.iter() { mark_seen(self, loc); }
        }

        // Compute field-of-view using recursive shadowcasting in hex grid
        // geometry.
        fn process(
            fov: &mut Fov, world: &World, range: uint,
            center: Location, begin: Angle, end: Angle) {
            if begin.radius > range { return; }

            let mut angle = begin;
            let group_opaque = world.is_opaque(center + angle.to_vec());
            while angle.is_below(end) {
                let loc = center + angle.to_vec();
                if world.is_opaque(loc) != group_opaque {
                    process(fov, world, range, center, angle, end);
                    // Terrain opaquity has changed, time to recurse.
                    if !group_opaque {
                        process(fov, world, range, center, begin.further(), angle.further());
                    }
                    return;
                }
                mark_seen(fov, loc);

                angle = angle.next();
            }

            if !group_opaque {
                process(fov, world, range, center, begin.further(), end.further());
            }
        }

        fn mark_seen(fov: &mut Fov, loc: Location) {
            fov.seen.insert(loc);
            fov.remembered.insert(loc);
        }
    }
}


#[deriving(PartialEq)]
struct Angle {
    pos: f32,
    radius: uint
}

impl Angle {
    pub fn new(pos: f32, radius: uint) -> Angle { Angle { pos: pos, radius: radius } }
    pub fn winding_index(self) -> int { (self.pos + 0.5).floor() as int }
    pub fn end_index(self) -> int { (self.pos + 0.5).ceil() as int }
    pub fn is_below(self, other: Angle) -> bool { self.winding_index() < other.end_index() }
    pub fn to_vec(self) -> Vector2<int> {
        if self.radius == 0 {
            return Vector2::new(0, 0);
        }

        let index = self.winding_index();

        let sector = index.mod_floor(&(self.radius as int * 6)) as uint / self.radius;
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
