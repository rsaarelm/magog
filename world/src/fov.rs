use calx_grid::{FovValue, HexGeom};
use euclid::Vector2D;
use location::Location;
use terraform::TerrainQuery;
use world::World;

#[derive(Clone)]
pub struct SightFov<'a> {
    w: &'a World,
    range: u32,
    pub origin: Location,
    is_edge: bool,
}

impl<'a> SightFov<'a> {
    pub fn new(w: &'a World, range: u32, origin: Location) -> SightFov<'a> {
        SightFov {
            w,
            range,
            origin,
            is_edge: false,
        }
    }
}

impl<'a> PartialEq for SightFov<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.w as *const World == other.w as *const World && self.range == other.range &&
        self.origin == other.origin && self.is_edge == other.is_edge
    }
}

impl<'a> Eq for SightFov<'a> {}

impl<'a> FovValue for SightFov<'a> {
    fn advance(&self, offset: Vector2D<i32>) -> Option<Self> {
        if offset.hex_dist() as u32 > self.range {
            return None;
        }

        if self.is_edge {
            return None;
        }

        let mut ret = self.clone();
        if let Some(dest) = self.w.visible_portal(self.origin + offset) {
            ret.origin = dest - offset;
        }

        if self.w.terrain(self.origin + offset).blocks_sight() {
            ret.is_edge = true;
        }

        Some(ret)
    }

    fn is_fake_isometric_wall(&self, offset: Vector2D<i32>) -> bool {
        self.w.terrain(self.origin + offset).is_wall()
    }
}
