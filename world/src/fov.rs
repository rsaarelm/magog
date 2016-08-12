use std::collections::HashSet;
use euclid::Point2D;
use std::iter::FromIterator;
use calx_grid::{FovValue, HexFov, HexGeom};
use world::World;
use query;
use location::Location;
use terrain;

#[derive(Clone)]
struct SightFov<'a> {
    w: &'a World,
    range: u32,
    origin: Location,
    prev_offset: Point2D<i32>,
}

impl<'a> PartialEq for SightFov<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.w as *const World == other.w as *const World && self.range == other.range &&
        self.origin == other.origin && self.prev_offset == other.prev_offset
    }
}

impl<'a> Eq for SightFov<'a> {}

impl<'a> FovValue for SightFov<'a> {
    fn advance(&self, offset: Point2D<i32>) -> Option<Self> {
        if offset.hex_dist() as u32 > self.range {
            return None;
        }

        if query::terrain(self.w, self.origin + self.prev_offset).blocks_sight() {
            return None;
        }

        let mut ret = self.clone();
        ret.prev_offset = offset;
        if let Some(dest) = query::visible_portal(self.w, self.origin + offset) {
            ret.origin = dest - offset;
        }

        Some(ret)
    }

    fn is_fake_isometric_wall(&self, offset: Point2D<i32>) -> bool {
        query::terrain(self.w, self.origin + offset).form == terrain::Form::Wall
    }
}

/// Return the field of view chart for visible tiles.
pub fn sight_fov(w: &World, origin: Location, range: u32) -> HashSet<Location> {
    let init = SightFov {
        w: w,
        range: range,
        origin: origin,
        prev_offset: Point2D::new(0, 0),
    };

    HashSet::from_iter(HexFov::new(init).map(|(pos, a)| a.origin + pos))
}
