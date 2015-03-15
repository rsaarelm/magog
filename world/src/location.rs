use std::ops::{Add};
use util::DijkstraNode;
use util::V2;
use util;
use dir6::Dir6;
use entity::Entity;
use terrain::TerrainType;
use geom::HexGeom;
use world;
use action;
use flags;
use ecs::{ComponentAccess};
use {Light, Biome};

/// Unambiguous location in the game world.
#[derive(Copy, Eq, PartialEq, Clone, Hash, PartialOrd, Ord, Debug, RustcEncodable, RustcDecodable)]
pub struct Location {
    pub x: i8,
    pub y: i8,
    // TODO: Add third dimension for multiple persistent levels.
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
            let n = util::noise(self.x as i32 + self.y as i32 * 57);
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

    /// Return the most significant entity at the location.
    pub fn main_entity(&self) -> Option<Entity> {
        let e = self.entities();
        // TODO: Actually make the result entity the most significant one (eg.
        // mob if there are both mobs and items).
        if e.len() > 0 { Some(e[0]) } else { None }
    }

    /// Return an item at the location that can be interacted with.
    pub fn top_item(&self) -> Option<Entity> {
        self.entities().iter().find(|e| e.can_be_picked_up()).map(|&x| x)
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
    pub fn v2_at(&self, other: Location) -> Option<V2<i32>> {
        // Return None for pairs on different floors if multi-floor support is
        // added.
        Some(V2(other.x as i32, other.y as i32) - V2(self.x as i32, self.y as i32))
    }

    /// Hex distance from this location to the other one, if applicable.
    pub fn distance_from(&self, other: Location) -> Option<i32> {
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
            1 => "Outside".to_string(),
            n => format!("Basement {}", n - 1)
        }
    }

    /// Light level for the location.
    pub fn light(&self) -> Light {
        if action::current_depth() == 1 {
            // Topside, full light.
            return Light::new(1.0);
        }
        if self.terrain().is_luminous() {
            return Light::new(1.0);
        }

        if let Some(d) = self.distance_from(flags::camera()) {
            let lum = 0.8 - d as f32 / 10.0;
            return Light::new(if lum >= 0.0 { lum } else { 0.0 });
        }
        return Light::new(1.0);
    }

    pub fn biome(&self) -> Biome {
        match world::with(|w| { w.area.biomes.get(self).map(|&x| x) }) {
            Some(b) => b,
            _ => Biome::Overland
        }
    }

    /// Try to find a nearby valid location if self doesn't satisfy predicate.
    pub fn spill<P: Fn<(Location,), Output=bool>>(&self, valid_pos: P) -> Option<Location> {
        if valid_pos(*self) { return Some(*self); }
        if let Some(loc) = Dir6::iter().map(|d| *self + d.to_v2()).find(|&x| valid_pos(x)) {
            return Some(loc);
        }
        None
    }
}

impl Add<V2<i32>> for Location {
    type Output = Location;
    fn add(self, other: V2<i32>) -> Location {
        Location::new(
            (self.x as i32 + other.0) as i8,
            (self.y as i32 + other.1) as i8)
    }
}

/// An abstract type that maps a 2D plane into game world Locations. This can
/// be just a straightforward mapping, or it can involve something exotic like
/// a non-Euclidean space where the lines from the Chart origin are raycast
/// through portals.
pub trait Chart: Add<V2<i32>, Output=Location> {}

impl Chart for Location {}

/// The other half of a Chart, mapping Locations into 2D plane positions, if a
/// mapping exists. It depends on the weirdness of a space how trivial this is
/// to do.
pub trait Unchart {
    fn chart_pos(&self, loc: Location) -> Option<V2<i32>>;
}

impl Unchart for Location {
    fn chart_pos(&self, loc: Location) -> Option<V2<i32>> {
        Some(V2(loc.x as i32 - self.x as i32, loc.y as i32 - self.y as i32))
    }
}

impl DijkstraNode for Location {
    fn neighbors(&self) -> Vec<Location> {
        Dir6::iter().map(|d| *self + d.to_v2()).collect()
    }
}
