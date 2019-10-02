//! Top level world generation logic

use crate::location::Location;
use crate::map::Map;
use crate::spec::{self, EntitySpawn, Spec};
use crate::terrain::Terrain;
use crate::vaults;
use crate::{Distribution, Rng};
use calx::{self, die, seeded_rng, CellVector, RngExt, WeightedChoice};
use euclid::{vec2, vec3, Vector3D};
use log::{debug, warn};
use rand::seq::SliceRandom;
use rand::Rng as _;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::ops::{Add, Deref, DerefMut};
use std::str::FromStr;
use std::sync::Arc;

pub const SECTOR_WIDTH: i32 = 38;
pub const SECTOR_HEIGHT: i32 = 18;

pub struct SectorSpace;
pub type SectorVector = Vector3D<i16, SectorSpace>;

/// Non-scrolling screen.
///
/// A sector represents a rectangular chunk of locations that fit on the visual screen. Sector
/// coordinates form their own sector space that tiles the location space with sectors.
#[derive(
    Copy, Clone, Eq, PartialEq, Default, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize,
)]
pub struct Sector {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

impl Add<SectorVector> for Sector {
    type Output = Sector;
    fn add(self, other: SectorVector) -> Sector {
        Sector {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sector {
    pub const fn new(x: i16, y: i16, z: i16) -> Sector { Sector { x, y, z } }

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

    /// Generate pseudorandom downstairs pos guaranteed not to collide with upstairs pos for this
    /// sector.
    pub fn downstairs_location(self, seed: u32) -> Location {
        // The trick: Split the sector into vertical strips. Use odd strips for odd z coordinate
        // floor's downstairs and even strips for even z coordinate floor's downstairs. This way
        // consecutive stairwells are always guaranteed not to end up on the same spot.
        //
        // Since stairwells have some architecture around them, also keep the strips with one cell
        // of padding between them. So we actually end up with
        //
        //     even: 1 + 4n
        //     odd:  3 + 4n
        //
        // These are using the rectangular (u, v) sector coordinates instead of the regular (x, y)
        // hex coordinates because the trick is formulated in terms of rectangular coordinate space
        // columns.

        // Bump for odd z floors.
        let u_offset = 1 + (self.z as i32).rem_euclid(2) * 2;

        let n = (SECTOR_WIDTH - 1) / 4;
        debug_assert!(n > 0);
        debug_assert!(SECTOR_HEIGHT > 6);

        let mut rng = seeded_rng(&(&seed, &self));
        let u = 4 * rng.gen_range(0, n) + u_offset;
        // Leave space to top and bottom so you can make a path from the stairwell. Stairs usually
        // have a vertical enclosure.
        let v = rng.gen_range(3, SECTOR_HEIGHT - 3);

        self.rect_coord_loc(u, v)
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Biome {
    Dungeon = 1,
    Grassland,
    Forest,
    Mountain,
    Desert,
    Water,
    City,
}

impl Default for Biome {
    fn default() -> Self { Biome::Water }
}

/// Specification for generating a Sector's map.
///
/// This serves as the top-level entry point to map generation routines.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SectorSpec {
    // TODO: Sectors can be predefined maps.
    // TODO: flags for blocked connection to N,E,W,S,up and down neighbor sectors
    // By default create path/stairs if adjacent sector exists.
    pub depth: i32,
    pub biome: Biome,
    pub west_wall: bool,
    pub south_wall: bool,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct WorldSkeleton(HashMap<Sector, SectorSpec>);

impl Deref for WorldSkeleton {
    type Target = HashMap<Sector, SectorSpec>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for WorldSkeleton {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl WorldSkeleton {
    pub fn dungeon_dive() -> WorldSkeleton {
        let mut ret = WorldSkeleton::default();
        for depth in 0..10 {
            let sector = Sector::new(0, 0, -(depth as i16));
            let spec = SectorSpec {
                depth,
                biome: Biome::Dungeon,
                ..Default::default()
            };
            ret.insert(sector, spec);
        }
        ret
    }

    pub fn overworld_sprawl() -> WorldSkeleton {
        use calx::{DenseTextMap, IntoPrefab};
        const OVERWORLD_MAP: &str = "
             ~ ~ ~ ~ ~ ~ ^ ^ ^ ^
             ~ ~ ~ ~ . % %|- - ^
             ~ ~ . # . . %L- - ^
             ~ . . . 0 . . .|- ^
             ~ . . . . % ._-_- ^
             ~ . . . . . . . . ^
             ~ . . . # # . . . ^
             ~ ~ . . # . . . . ^
             ~ ~ . . . . . . . ^
             ~ ~ ~ ~ ~ ~ ~ ^ ^ ^";

        let wide_map: HashMap<CellVector, char> = DenseTextMap(OVERWORLD_MAP)
            .into_prefab()
            .expect("Invalid overworld map");

        let origin = wide_map
            .iter()
            .find(|(_, &c)| c == '0')
            .map(|(&p, _)| p)
            .expect("No origin sector defined");

        let mut map = HashMap::new();

        for (&wide_p, &c) in &wide_map {
            if c == ' ' {
                continue;
            }

            let wide_p = wide_p - origin;
            let (p, is_side) = (CellVector::new(wide_p.x / 2, wide_p.y), wide_p.x % 2 != 0);

            // (is_blocked_west, is_blocked_south, sector_biome)
            let entry = map.entry(p).or_insert((false, false, Biome::Water));

            if is_side {
                match c {
                    '|' => (*entry).0 = true,
                    'L' => {
                        (*entry).0 = true;
                        (*entry).1 = true;
                    }
                    '_' => (*entry).1 = true,
                    _ => panic!("Bad side char {}", c),
                }
            } else {
                match c {
                    '~' => (*entry).2 = Biome::Water,
                    '-' => (*entry).2 = Biome::Desert,
                    '.' => (*entry).2 = Biome::Grassland,
                    '0' => (*entry).2 = Biome::Grassland,
                    '%' => (*entry).2 = Biome::Forest,
                    '#' => (*entry).2 = Biome::City,
                    '^' => (*entry).2 = Biome::Mountain,
                    _ => panic!("Unknown biome char {}", c),
                }
            }
        }

        // Legend (sector centers):
        // 0: player start sector
        // ~: sea
        // .: grassland
        // %: forest
        // -: desert
        // ^: mountain
        // #: city
        //
        // Legend (sector edges):
        // I, L, _: Edges between sectors to left, right and lower right

        let mut ret = WorldSkeleton::default();
        // Overworld
        for (p, (west_wall, south_wall, biome)) in &map {
            let depth = if *p == vec2(0, 0) {
                // No spawns in entrance sector.
                -1
            } else {
                (p.x.abs() + p.y.abs()) / 2
            };
            let sector = Sector::new(p.x as i16, p.y as i16, 0);
            let spec = SectorSpec {
                depth,
                biome: *biome,
                west_wall: *west_wall,
                south_wall: *south_wall,
            };
            ret.insert(sector, spec);
        }

        // Dungeons
        for depth in 0..10 {
            let sector = Sector::new(0, 0, -(depth as i16 + 1));
            let spec = SectorSpec {
                depth,
                biome: Biome::Dungeon,
                ..Default::default()
            };
            ret.insert(sector, spec);
        }

        ret
    }
}

/// Generate the map for a sector given the 3D world skeleton.
///
/// Note that this function does not take a rng. The idea is that map generation should be
/// perfectly deterministic given a world seed and the sector position, so new sectors can be
/// lazily generated at any point of the game.
pub fn generate(seed: u32, pos: Sector, world_skeleton: &WorldSkeleton) -> Map {
    ConnectedSectorSpec::new(seed, pos, world_skeleton).sample(&mut calx::seeded_rng(&(seed, pos)))
}

/// Wrapper for `SectorSpec` with references to neighboring sectors.
///
/// This is needed for map generation where connections or terrain transition tiles on the edge
/// will be placed based on the neighboring sectors. Map generation will try to build connective
/// pathways to traversable neighboring sectors unless the central spec specifically forbids it.
pub struct ConnectedSectorSpec<'a> {
    pub seed: u32,
    pub sector: Sector,
    pub spec: &'a SectorSpec,
    pub north: Option<&'a SectorSpec>,
    pub east: Option<&'a SectorSpec>,
    pub south: Option<&'a SectorSpec>,
    pub west: Option<&'a SectorSpec>,
    pub up: Option<&'a SectorSpec>,
    pub down: Option<&'a SectorSpec>,
}

impl<'a> ConnectedSectorSpec<'a> {
    pub fn new(seed: u32, sector: Sector, world_skeleton: &'a WorldSkeleton) -> Self {
        ConnectedSectorSpec {
            seed,
            sector,
            spec: world_skeleton.get(&sector).unwrap(),
            north: world_skeleton.get(&(sector + vec3(0, -1, 0))),
            east: world_skeleton.get(&(sector + vec3(1, 0, 0))),
            south: world_skeleton.get(&(sector + vec3(0, 1, 0))),
            west: world_skeleton.get(&(sector + vec3(-1, 0, 0))),
            up: world_skeleton.get(&(sector + vec3(0, 0, 1))),
            down: world_skeleton.get(&(sector + vec3(0, 0, -1))),
        }
    }
}

impl<'a> Deref for ConnectedSectorSpec<'a> {
    type Target = SectorSpec;

    fn deref(&self) -> &SectorSpec { self.spec }
}

impl<'a> Distribution<Map> for ConnectedSectorSpec<'a> {
    fn sample(&self, rng: &mut Rng) -> Map {
        use Biome::*;
        match self.biome {
            Dungeon => self.build_dungeon(rng),
            Grassland => self.build_grassland(rng),
            Forest => self.base_map(Terrain::Tree), // TODO
            Mountain => self.base_map(Terrain::Rock),
            Desert => self.base_map(Terrain::Sand), // TODO
            Water => self.base_map(Terrain::Water),
            City => self.base_map(Terrain::Ground), // TODO
        }
    }
}

impl<'a> ConnectedSectorSpec<'a> {
    fn dungeon_gen(&self, rng: &mut Rng) -> Result<Map, Box<dyn Error>> {
        // TODO: Connect to side levels if they exist

        debug!("Starting mapgen");
        let mut map = self.dungeon_base_map();

        self.place_stairs(rng, &mut map)?;

        loop {
            let room: Room = self.sample(rng);
            debug!("Adding room");
            if map.place_room(rng, &*room.0).is_err() {
                break;
            }
        }

        if let Some(map) = map.join_disjoint_regions(rng) {
            Ok(map)
        } else {
            die!("Failed to join map");
        }
    }

    fn place_stairs(&self, rng: &mut Rng, map: &mut Map) -> Result<(), Box<dyn Error>> {
        // TODO: Biome affects vault distribution
        if self.up.is_some() {
            let room: Entrance = self.sample(rng);
            debug!("Placing upstairs");
            map.place_room(rng, &*room.0)?;
        }

        if self.down.is_some() {
            // TODO: Make exit use a sampled type like Entrance does
            debug!("Placing downstairs");
            let room = vaults::EXITS.choose(rng).unwrap();
            map.place_room(rng, &*room)?;
        }
        Ok(())
    }

    fn build_dungeon(&self, rng: &mut Rng) -> Map {
        const NUM_RETRIES: usize = 16;

        if let Ok(map) = calx::retry_gen(NUM_RETRIES, rng, |rng| self.dungeon_gen(rng)) {
            map
        } else {
            // Fallback, couldn't generate map, let's do something foolproof.
            warn!("Repeated dungeon generation failure, falling back to bigroom");
            self.build_bigroom(rng)
        }
    }

    fn build_bigroom(&self, rng: &mut Rng) -> Map {
        let mut map = self.dungeon_base_map();

        self.place_stairs(rng, &mut map).unwrap();

        for p in map.find_positions(|_, _| true) {
            map.dig(p);
        }

        for &pos in &map.open_ground() {
            if let Some(spawn) = self.sample(rng) {
                map.push_spawn(pos, spawn);
            }
        }

        map
    }

    fn downstairs_pos(&self) -> Option<CellVector> {
        self.down.map(|_| {
            self.sector
                .origin()
                .v2_at(self.sector.downstairs_location(self.seed))
                .unwrap()
        })
    }

    fn upstairs_pos(&self) -> Option<CellVector> {
        self.up.map(|_| {
            let mut upstairs_pos = (self.sector + vec3(0, 0, 1)).downstairs_location(self.seed);
            upstairs_pos.z -= 1;
            // Offset it so that the exits line up nicer.
            upstairs_pos.x -= 1;
            upstairs_pos.y -= 1;
            self.sector.origin().v2_at(upstairs_pos).unwrap()
        })
    }

    fn place_stairwells(&self, map: &mut Map) {
        if let Some(down_pos) = self.downstairs_pos() {
            map.set_terrain(down_pos, Terrain::Downstairs);
            debug!("Downstairs for {:?} at {:?}", self.sector, down_pos);
        }

        if let Some(up_pos) = self.upstairs_pos() {
            map.set_terrain(up_pos, Terrain::Upstairs);
            debug!("Upstairs for {:?} at {:?}", self.sector, up_pos);
        }
    }

    fn dungeon_base_map(&self) -> Map {
        let mut ret = Map::new_base(
            Terrain::Rock,
            Sector::points()
                .filter(|p| !Location::new(p.x as i16, p.y as i16, 0).is_next_to_diagonal_sector()),
        );
        self.place_stairwells(&mut ret);
        ret
    }

    fn base_map(&self, terrain: Terrain) -> Map {
        let mut ret = Map::new_base(terrain, Sector::points());
        self.place_stairwells(&mut ret);
        ret
    }

    fn build_grassland(&self, rng: &mut Rng) -> Map {
        let mut map = self.base_map(Terrain::Grass);
        self.place_stairs(rng, &mut map).unwrap();

        for &pos in &map.open_ground() {
            if let Some(spawn) = self.sample(rng) {
                map.push_spawn(pos, spawn);
            }
        }

        map
    }

    fn can_spawn(&self, spec: &dyn Spec) -> bool {
        spec.min_depth() <= self.depth && (spec.habitat() & (1 << self.biome as u64)) != 0
    }
}

impl Distribution<EntitySpawn> for ConnectedSectorSpec<'_> {
    fn sample(&self, rng: &mut Rng) -> EntitySpawn {
        let item = spec::iter_specs()
            .weighted_choice(rng, |item| {
                if item.rarity() == 0.0 || !self.can_spawn(&**item) {
                    0.0
                } else {
                    1.0 / item.rarity()
                }
            })
            .unwrap();

        EntitySpawn::from_str(item.name()).unwrap()
    }
}

/// Include spawn density of entities, can be run over all open cells.
///
/// XXX: You maybe want something smarter than this to handle clustering of mobs etc.
impl Distribution<Option<EntitySpawn>> for ConnectedSectorSpec<'_> {
    fn sample(&self, rng: &mut Rng) -> Option<EntitySpawn> {
        use Biome::*;
        if self.depth == -1 {
            return None;
        }

        let spawn_one_in = match self.biome {
            Dungeon => 10,
            _ => 100,
        };

        if rng.one_chance_in(spawn_one_in) {
            Some(self.sample(rng))
        } else {
            None
        }
    }
}

struct Entrance(Arc<Map>);

impl Distribution<Entrance> for ConnectedSectorSpec<'_> {
    fn sample(&self, rng: &mut Rng) -> Entrance {
        Entrance(vaults::ENTRANCES.choose(rng).unwrap().clone())
    }
}

struct Room(Arc<Map>);

impl Distribution<Room> for ConnectedSectorSpec<'_> {
    fn sample(&self, rng: &mut Rng) -> Room {
        if rng.one_chance_in(12) {
            // Make a vault sometimes.
            Room(vaults::VAULTS.choose(rng).unwrap().clone())
        } else {
            // Make a procgen room normally.
            let mut map = Map::new_plain_room(rng);
            for &pos in &map.open_ground() {
                if let Some(spawn) = self.sample(rng) {
                    map.push_spawn(pos, spawn);
                }
            }

            Room(Arc::new(map))
        }
    }
}

struct Exit(Arc<Map>);

impl Distribution<Exit> for ConnectedSectorSpec<'_> {
    fn sample(&self, rng: &mut Rng) -> Exit { Exit(vaults::EXITS.choose(rng).unwrap().clone()) }
}

#[cfg(test)]
mod test {
    use super::{Sector, SECTOR_HEIGHT, SECTOR_WIDTH};
    use euclid::vec3;

    #[test]
    fn test_rect_space() {
        let s = Sector::new(0, 0, 0);
        for v in 0..SECTOR_HEIGHT {
            for u in 0..SECTOR_WIDTH {
                let loc = s.rect_coord_loc(u, v);
                assert!(s.iter().find(|x| x == &loc).is_some());
            }
        }
    }

    #[test]
    fn test_stair_locations() {
        for z in -1000..1000 {
            let s = Sector::new(0, 0, z);
            let loc = s.downstairs_location(123);

            // Locations must be placed inside sector.
            assert!(s.iter().find(|x| x == &loc).is_some());

            // Location must not collide with the other stair location.
            let mut upstairs_loc = (s + vec3(0, 0, 1)).downstairs_location(123);
            upstairs_loc.z = loc.z;
            assert!(loc.distance_from(upstairs_loc).unwrap() > 1);
        }
    }
}
