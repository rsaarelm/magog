use crate::location::Location;
use calx::CellVector;
use serde_derive::{Deserialize, Serialize};

pub const SECTOR_WIDTH: i32 = 38;
pub const SECTOR_HEIGHT: i32 = 18;

/// Non-scrolling screen.
///
/// A sector represents a rectangular chunk of locations that fit on the visual screen. Sector
/// coordinates form their own sector space that tiles the location space with sectors.
#[derive(Copy, Eq, PartialEq, Clone, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct Sector {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

impl Sector {
    pub fn new(x: i16, y: i16, z: i16) -> Sector { Sector { x, y, z } }

    pub fn origin(self) -> Location { self.rect_coord_loc(0, 0) }

    pub fn rect_coord_loc(self, u: i32, v: i32) -> Location {
        Location::from_rect_coords(
            self.x as i32 * SECTOR_WIDTH + u,
            self.y as i32 * SECTOR_HEIGHT + v,
            self.z,
        )
    }

    /// Center location for this sector.
    ///
    /// Usually you want the camera positioned here.
    pub fn center(self) -> Location {
        // XXX: If the width/height are even (as they currently are), there isn't a centered cell.
        self.rect_coord_loc(SECTOR_WIDTH / 2 - 1, SECTOR_HEIGHT / 2 - 1)
    }

    pub fn iter(self) -> impl Iterator<Item = Location> {
        let n = SECTOR_WIDTH * SECTOR_HEIGHT;
        let pitch = SECTOR_WIDTH;
        (0..n).map(move |i| self.rect_coord_loc(i % pitch, i / pitch))
    }

    /// Iterate offset points for a generic `Sector`.
    pub fn points() -> impl Iterator<Item = CellVector> {
        let sector = Sector::new(0, 0, 0);
        let sector_origin = sector.origin();
        sector
            .iter()
            .map(move |loc| sector_origin.v2_at(loc).unwrap())
    }

    pub fn taxicab_distance(self, other: Sector) -> i32 {
        ((self.x as i32) - (other.x as i32)).abs()
            + ((self.y as i32) - (other.y as i32)).abs()
            + ((self.z as i32) - (other.z as i32)).abs()
    }
}
