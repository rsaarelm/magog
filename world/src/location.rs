use calx::dijkstra;
use calx::V2;
use dir6::Dir6;
use entity::Entity;
use terrain::TerrainType;
use geom::HexGeom;
use world;
use action;

/// Unambiguous location in the game world.
#[deriving(Copy, Eq, PartialEq, Clone, Hash, Show, Encodable, Decodable)]
pub struct Location {
    pub x: i8,
    pub y: i8,
    // TODO: Add third dimension for multiple persistent levels.
}

fn noise(n: int) -> f32 {
    // TODO: Move to an utilities library.
    let n = (n << 13) ^ n;
    let m = (n * (n * n * 15731 + 789221) + 1376312589) & 0x7fffffff;
    1.0 - m as f32 / 1073741824.0
}

impl Location {
    pub fn new(x: i8, y: i8) -> Location { Location { x: x, y: y } }

    /// Return terrain at the location.
    pub fn terrain(&self) -> TerrainType {
        let mut ret = world::with(|w| w.area.terrain(*self));
        // Mobs standing on doors make the doors open.
        if ret == TerrainType::Door && self.has_mobs() {
            ret = TerrainType::OpenDoor;
        }
        // Grass is only occasionally fancy.
        // TODO: Make variant tiles into a generic method.
        if ret == TerrainType::Grass {
            let n = noise(self.x as int + self.y as int * 57);
            if n > 0.85 {
                ret = TerrainType::Grass2;
            }
        }
        ret
    }

    pub fn blocks_sight(&self) -> bool {
        self.terrain().blocks_sight()
    }

    pub fn blocks_walk(&self) -> bool {
        if self.terrain().blocks_walk() { return true; }
        if self.entities().iter().any(|e| e.blocks_walk()) {
            return true;
        }
        false
    }

    pub fn entities(&self) -> Vec<Entity> {
        world::with(|w| w.spatial.entities_at(*self))
    }

    pub fn has_entities(&self) -> bool { !self.entities().is_empty() }

    pub fn has_mobs(&self) -> bool {
        self.entities().iter().any(|e| e.is_mob())
    }

    /// Returns the mob at the given location, if any. The assumption is that
    /// there is always a single primary mob in locations that contain
    /// multiple mobs that can be selected as the return value.
    pub fn mob_at(&self) -> Option<Entity> {
        for &e in self.entities().iter() {
            if e.is_mob() {
                return Some(e);
            }
        }
        None
    }

    /// Vector pointing from this location into the other one if the locations
    /// are on the same Euclidean plane.
    pub fn v2_at(&self, other: Location) -> Option<V2<int>> {
        // Return None for pairs on different floors if multi-floor support is
        // added.
        Some(V2(other.x as int, other.y as int) - V2(self.x as int, self.y as int))
    }

    /// Hex distance from this location to the other one, if applicable.
    pub fn distance_from(&self, other: Location) -> Option<int> {
        if let Some(v) = self.v2_at(other) { Some(v.hex_dist()) } else { None }
    }

    pub fn dir6_towards(&self, other: Location) -> Option<Dir6> {
        if let Some(v) = self.v2_at(other) { Some(Dir6::from_v2(v)) } else { None }
    }

    /// Return the status of the location in the player's field of view.
    /// Returns None if the location is unexplored.
    pub fn fov_status(&self) -> Option<::FovStatus> {
        if let Some(p) = action::player() {
            match world::with(|w| {
                if let Some(ref mm) = w.map_memories().get(p) {
                    Ok (if mm.seen.contains(self) {
                        Some(::FovStatus::Seen)
                    } else if mm.remembered.contains(self) {
                        Some(::FovStatus::Remembered)
                    } else {
                        None
                    })
                } else {
                    Err(())
                }
            }) {
                Ok(x) => return x,
                _ => ()
            };
        }
        // Just show everything by default.
        Some(::FovStatus::Seen)
    }

    /// Area name for the location.
    pub fn name(&self) -> String {
        match action::current_depth() {
            0 => "Limbo".to_string(),
            1 => "Overworld".to_string(),
            n => format!("Dungeon {}", n - 1)
        }
    }
}

impl Add<V2<int>, Location> for Location {
    fn add(self, other: V2<int>) -> Location {
        Location::new(
            (self.x as int + other.0) as i8,
            (self.y as int + other.1) as i8)
    }
}

/// An abstract type that maps a 2D plane into game world Locations. This can
/// be just a straightforward mapping, or it can involve something exotic like
/// a non-Euclidean space where the lines from the Chart origin are raycast
/// through portals.
pub trait Chart: Add<V2<int>, Location> {}

impl Chart for Location {}

/// The other half of a Chart, mapping Locations into 2D plane positions, if a
/// mapping exists. It depends on the weirdness of a space how trivial this is
/// to do.
pub trait Unchart {
    fn chart_pos(&self, loc: Location) -> Option<V2<int>>;
}

impl Unchart for Location {
    fn chart_pos(&self, loc: Location) -> Option<V2<int>> {
        Some(V2(loc.x as int - self.x as int, loc.y as int - self.y as int))
    }
}

impl dijkstra::Node for Location {
    fn neighbors(&self) -> Vec<Location> {
        Dir6::iter().map(|d| *self + d.to_v2()).collect()
    }
}
