use euclid::Point2D;
use calx_grid::{FovValue, HexGeom};
use world::World;
use location::Location;
use query::Query;
use terrain;

#[derive(Clone)]
pub struct SightFov<'a> {
    w: &'a World,
    range: u32,
    pub origin: Location,
    prev_offset: Point2D<i32>,
}

impl<'a> SightFov<'a> {
    pub fn new(w: &'a World, range: u32, origin: Location) -> SightFov<'a> {
        SightFov {
            w: w,
            range: range,
            origin: origin,
            prev_offset: Point2D::new(0, 0),
        }
    }
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

        if self.w.terrain(self.origin + self.prev_offset).blocks_sight() {
            return None;
        }

        let mut ret = self.clone();
        ret.prev_offset = offset;
        if let Some(dest) = self.w.visible_portal(self.origin + offset) {
            ret.origin = dest - offset;
        }

        Some(ret)
    }

    fn is_fake_isometric_wall(&self, offset: Point2D<i32>) -> bool {
        self.w.terrain(self.origin + offset).form == terrain::Form::Wall
    }
}

