//! Top level world generation logic

use crate::{
    location::Location,
    map::{Map, MapCell},
    spec::{self, EntitySpawn, Spec},
    terrain::Terrain,
    vaults, {Distribution, Rng},
};
use calx::{
    die, project, seeded_rng, CellSpace, CellVector, ProjectVec, RngExt, Space, StaggeredHexSpace,
    WeightedChoice,
};
use euclid::{vec2, vec3, Vector2D};
use lazy_static::lazy_static;
use log::{debug, warn};
use rand::seq::SliceRandom;
use rand::Rng as _;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::ops::{Add, Deref, DerefMut};
use std::str::FromStr;
use std::sync::Arc;

pub const SECTOR_HEX_SIDE: i32 = 20;

pub const SECTOR_HEIGHT: i32 = SECTOR_HEX_SIDE;
pub const SECTOR_WIDTH: i32 = SECTOR_HEX_SIDE * 2;

pub struct SectorSpace;
impl Space for SectorSpace {
    type T = i32;
}
pub type SectorVec = euclid::Vector3D<i16, SectorSpace>;

impl project::From<CellSpace> for SectorSpace {
    fn vec_from(vec: Vector2D<<CellSpace as Space>::T, CellSpace>) -> Vector2D<Self::T, Self> {
        // Macro-hex sectors (uv) in cell space (xy)
        //
        //     o-------+===-->x
        //     |\  00  | 11
        //     | \ .   |   .
        //     |  \    |
        //     | . +---+   *
        //     |   |01  \
        //     |-10|   . \ .
        //     |   |      \
        //     +---+ . * . +
        //     I-11 \      |
        //     I   . \ .   |
        //     I      \    |
        //     + . * . +---+
        //     |
        //     v
        //     y
        //
        // The above rectangle produces a repeating pattern of terrain hexes that lines up
        // perfectly with the sector grid.

        lazy_static! {
            static ref SECTOR_OFFSETS: Vec<Vec<Vector2D<i32, SectorSpace>>> = {
                assert!(SECTOR_HEX_SIDE == 20, "TODO: Adjustable sector hex size");

                fn to_signed(u2: u32) -> i32 {
                    if u2 > 1 {
                        -4 + u2 as i32
                    } else {
                        u2 as i32
                    }
                }

                fn delta(c: char) -> Vector2D<i32, SectorSpace> {
                    let c = c.to_digit(16).expect("invalid pattern");
                    vec2(to_signed(c % 4), to_signed(c / 4))
                }

                // TODO: Derive the pattern from SECTOR_HEX_SIDE procedurally instead of having
                // the ASCII art blob.

                // Hex char table, encode x and y coords for the HexVector offset as 2-bit 2's
                // complement signed integers (-2 to 1). x is low 2 bits, y is high 2 bits.
                const PATTERN: &str = "\
000000000000000000000000000000000000000011111111111111111111
300000000000000000000000000000000000000055555555555555555555
330000000000000000000000000000000000000055555555555555555555
333000000000000000000000000000000000000055555555555555555555
333300000000000000000000000000000000000055555555555555555555
333330000000000000000000000000000000000055555555555555555555
333333000000000000000000000000000000000055555555555555555555
333333300000000000000000000000000000000055555555555555555555
333333330000000000000000000000000000000055555555555555555555
333333333000000000000000000000000000000055555555555555555555
333333333300000000000000000000000000000055555555555555555555
333333333330000000000000000000000000000055555555555555555555
333333333333000000000000000000000000000055555555555555555555
333333333333300000000000000000000000000055555555555555555555
333333333333330000000000000000000000000055555555555555555555
333333333333333000000000000000000000000055555555555555555555
333333333333333300000000000000000000000055555555555555555555
333333333333333330000000000000000000000055555555555555555555
333333333333333333000000000000000000000055555555555555555555
333333333333333333300000000000000000000055555555555555555555
333333333333333333330000000000000000000055555555555555555555
333333333333333333334444444444444444444445555555555555555555
333333333333333333334444444444444444444444555555555555555555
333333333333333333334444444444444444444444455555555555555555
333333333333333333334444444444444444444444445555555555555555
333333333333333333334444444444444444444444444555555555555555
333333333333333333334444444444444444444444444455555555555555
333333333333333333334444444444444444444444444445555555555555
333333333333333333334444444444444444444444444444555555555555
333333333333333333334444444444444444444444444444455555555555
333333333333333333334444444444444444444444444444445555555555
333333333333333333334444444444444444444444444444444555555555
333333333333333333334444444444444444444444444444444455555555
333333333333333333334444444444444444444444444444444445555555
333333333333333333334444444444444444444444444444444444555555
333333333333333333334444444444444444444444444444444444455555
333333333333333333334444444444444444444444444444444444445555
333333333333333333334444444444444444444444444444444444444555
333333333333333333334444444444444444444444444444444444444455
333333333333333333334444444444444444444444444444444444444445
333333333333333333334444444444444444444444444444444444444444
777777777777777777777444444444444444444444444444444444444444
777777777777777777777744444444444444444444444444444444444444
777777777777777777777774444444444444444444444444444444444444
777777777777777777777777444444444444444444444444444444444444
777777777777777777777777744444444444444444444444444444444444
777777777777777777777777774444444444444444444444444444444444
777777777777777777777777777444444444444444444444444444444444
777777777777777777777777777744444444444444444444444444444444
777777777777777777777777777774444444444444444444444444444444
777777777777777777777777777777444444444444444444444444444444
777777777777777777777777777777744444444444444444444444444444
777777777777777777777777777777774444444444444444444444444444
777777777777777777777777777777777444444444444444444444444444
777777777777777777777777777777777744444444444444444444444444
777777777777777777777777777777777774444444444444444444444444
777777777777777777777777777777777777444444444444444444444444
777777777777777777777777777777777777744444444444444444444444
777777777777777777777777777777777777774444444444444444444444
777777777777777777777777777777777777777444444444444444444444";

                PATTERN.lines().map(|line| line.chars().map(delta).collect()).collect()
            };
        }

        const SECTOR_PATTERN_W: i32 = SECTOR_HEX_SIDE * 3;
        const SECTOR_PATTERN_H: i32 = SECTOR_HEX_SIDE * 3;

        let (pattern_x, pattern_y) = (
            vec.x.div_euclid(SECTOR_PATTERN_W),
            vec.y.div_euclid(SECTOR_PATTERN_H),
        );
        let (x, y) = (
            vec.x.rem_euclid(SECTOR_PATTERN_W),
            vec.y.rem_euclid(SECTOR_PATTERN_H),
        );
        let offset = SECTOR_OFFSETS[y as usize][x as usize];

        vec2(2 * pattern_x - pattern_y, pattern_x + pattern_y) + offset
    }
}

impl project::From<SectorSpace> for CellSpace {
    fn vec_from(vec: Vector2D<<SectorSpace as Space>::T, SectorSpace>) -> Vector2D<Self::T, Self> {
        let a = SECTOR_HEX_SIDE;
        vec2(a * vec.x + a * vec.y, -a * vec.x + 2 * a * vec.y)
    }
}

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

impl Add<SectorVec> for Sector {
    type Output = Sector;
    fn add(self, other: SectorVec) -> Sector {
        Sector {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Add<euclid::Vector2D<i32, SectorSpace>> for Sector {
    type Output = Sector;
    fn add(self, other: euclid::Vector2D<i32, SectorSpace>) -> Sector {
        Sector {
            x: self.x + other.x as i16,
            y: self.y + other.y as i16,
            z: self.z,
        }
    }
}

impl From<Location> for Sector {
    fn from(loc: Location) -> Self {
        Sector::new(0, 0, loc.z) + CellVector::from(loc).project::<SectorSpace>()
    }
}

impl From<Sector> for Vector2D<i32, SectorSpace> {
    fn from(sec: Sector) -> Self { vec2(sec.x as i32, sec.y as i32) }
}

impl Sector {
    pub const fn new(x: i16, y: i16, z: i16) -> Sector { Sector { x, y, z } }

    /// Center location for this sector.
    ///
    /// Default camera position.
    pub fn center(self) -> Location { Location::from(self) + vec2(SECTOR_HEX_SIDE - 1, 0) }

    pub fn iter(self) -> impl Iterator<Item = Location> {
        let origin = Location::from(self);
        Sector::shape().map(move |p| origin + p)
    }

    /// Yield the points that form the shape of the origin `Sector`.
    pub fn shape() -> impl Iterator<Item = CellVector> {
        ((-SECTOR_HEX_SIDE + 1)..(SECTOR_HEX_SIDE + 1))
            .flat_map(move |y| (0..(SECTOR_HEX_SIDE * 2)).map(move |x| CellVector::new(x, y)))
            .filter(|p| p.project::<SectorSpace>() == vec2(0, 0))
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

        Location::from(self) + vec2::<i32, StaggeredHexSpace>(u, v).project()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub enum SectorDir {
    Northeast = 0,
    East,
    Southeast,
    Southwest,
    West,
    Northwest,
    Up,
    Down,
}

impl From<SectorDir> for SectorVec {
    fn from(dir: SectorDir) -> Self {
        use SectorDir::*;
        match dir {
            Northeast => vec2(0, -1).to_3d(),
            East => vec2(1, 0).to_3d(),
            Southeast => vec2(1, 1).to_3d(),
            Southwest => vec2(0, 1).to_3d(),
            West => vec2(-1, 0).to_3d(),
            Northwest => vec2(-1, -1).to_3d(),
            Up => vec3(0, 0, 1),
            Down => vec3(0, 0, -1),
        }
    }
}

/// Parts of a hex `Sector`. The `CenterRectangle` part corresponds to the game screen. The
/// triangles are the parts of the hex above and below that.
pub enum SectorPart {
    NorthTriangle,
    CenterRectangle,
    SouthTriangle,
}

impl From<CellVector> for SectorPart {
    fn from(vec: CellVector) -> Self {
        if vec.x < -vec.y {
            SectorPart::NorthTriangle
        } else if vec.x > SECTOR_HEX_SIDE * 2 - 2 - vec.y {
            SectorPart::SouthTriangle
        } else {
            SectorPart::CenterRectangle
        }
    }
}

/// Herringbone Wang tiles space
///
/// Tiles with even x are horizontal, tiles with odd x are vertical.
pub struct HerringboneSpace;
impl Space for HerringboneSpace {
    type T = i32;
}
pub type HerringboneVector = euclid::Vector2D<i32, HerringboneSpace>;

pub const HERRINGBONE_SIZE: i32 = 11;

impl project::From<CellSpace> for HerringboneSpace {
    fn vec_from(vec: Vector2D<<CellSpace as Space>::T, CellSpace>) -> Vector2D<Self::T, Self> {
        const HERRINGBONE_PATTERN_W: i32 = HERRINGBONE_SIZE * 4;
        const HERRINGBONE_PATTERN_H: i32 = HERRINGBONE_SIZE * 4;

        lazy_static! {
            static ref HERRINGBONE_OFFSETS: Vec<Vec<HerringboneVector>> = {
                fn to_signed(u2: u32) -> i32 {
                    if u2 > 1 {
                        -4 + u2 as i32
                    } else {
                        u2 as i32
                    }
                }

                fn delta(c: char) -> HerringboneVector {
                    let c = c.to_digit(16).expect("invalid pattern");
                    // Table values are offset to fit into the -2, 1 range, correct this here.
                    vec2(to_signed(c % 4), to_signed(c / 4)) + vec2(0, 2)
                }

                // Hex char table, encode x and y coords for the HexVector offset as 2-bit 2's
                // complement signed integers (-2 to 1). x is low 2 bits, y is high 2 bits.
                const PATTERN: &str = "\
889d
fccd
f300
2374";

                PATTERN.lines().map(|line| line.chars().map(delta).collect()).collect()
            };
        }

        let (pattern_x, pattern_y) = (
            vec.x.div_euclid(HERRINGBONE_PATTERN_W),
            vec.y.div_euclid(HERRINGBONE_PATTERN_H),
        );
        let (x, y) = (
            vec.x.rem_euclid(HERRINGBONE_PATTERN_W),
            vec.y.rem_euclid(HERRINGBONE_PATTERN_H),
        );
        let offset =
            HERRINGBONE_OFFSETS[(y / HERRINGBONE_SIZE) as usize][(x / HERRINGBONE_SIZE) as usize];

        vec2(2 * pattern_x - 2 * pattern_y, pattern_x + 3 * pattern_y) + offset
    }
}

impl project::From<HerringboneSpace> for CellSpace {
    fn vec_from(
        vec: Vector2D<<HerringboneSpace as Space>::T, HerringboneSpace>,
    ) -> Vector2D<Self::T, Self> {
        let a = HERRINGBONE_SIZE;
        // Uneven step pattern for x
        let sx = ((vec.x as f32 * 3.0) / 2.0).ceil() as i32;
        let sx2 = (vec.x as f32 / 2.0).ceil() as i32;
        vec2(a * sx + a * vec.y, -a * sx2 + a * vec.y)
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
    fn default() -> Self { Biome::Dungeon }
}

impl Biome {
    /// Return terrain for the biome at a given position.
    ///
    /// Wilderness biomes produce useful terrain via just this function. Dungeon terrains will just
    /// produce solid rock and must be generated with a separate map generator.
    pub fn terrain_at(self, seed: u32, loc: Location) -> Terrain {
        use Biome::*;

        // Get the tile-less ones out of the way.
        // XXX: Should Dungeon and City have herringbone sets too?
        match self {
            Dungeon => return Terrain::Rock,
            Water => return Terrain::Water,
            City => return Terrain::Ground,
            Mountain => return Terrain::Rock,
            _ => {}
        }

        let pos: CellVector = vec2(loc.x as i32, loc.y as i32);
        let chunk = pos.project::<HerringboneSpace>();

        let map = {
            let (horiz, vert) = match self {
                Grassland => (&*vaults::GRASS_HORIZ, &*vaults::GRASS_VERT),
                Forest => (&*vaults::FOREST_HORIZ, &*vaults::FOREST_VERT),
                Desert => (&*vaults::DESERT_HORIZ, &*vaults::DESERT_VERT),
                _ => panic!("Unsupported biome {:?}", self),
            };
            let mut rng = calx::seeded_rng(&(seed, chunk));
            if (chunk.x % 2) == 0 {
                horiz.choose(&mut rng).unwrap().clone()
            } else {
                vert.choose(&mut rng).unwrap().clone()
            }
        };

        let offset = pos - chunk.project();
        let cell = map
            .get(offset)
            .unwrap_or_else(|| panic!("No offset {:?} in herringbone chunk at {:?}", offset, loc));
        cell.terrain
    }
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
        // FIXME
        use calx::IntoPrefab;
        const OVERWORLD_MAP: &str = "
             ~ ~ ~ ~ ~ ~ ^ ^ ^ ^
            ~ ~ ~ ~ . % % - - ^
             ~ ~ . # . . % - - ^
            ~ . . .[.]. . . - ^
             ~ . . . . % . - - ^
            ~ . . . . . . . . ^
             ~ . . . # # . . . ^
            ~ ~ . . # . . . . ^
             ~ ~ . . . . . . . ^
            ~ ~ ~ ~ ~ ~ ~ ^ ^ ^";

        let map: HashMap<CellVector, Biome> = OVERWORLD_MAP
            .into_prefab::<HashMap<CellVector, char>>()
            .expect("Invalid overworld map")
            .into_iter()
            .map(|(p, c)| {
                (
                    p,
                    match c {
                        '~' => Biome::Water,
                        '-' => Biome::Desert,
                        '.' => Biome::Grassland,
                        '%' => Biome::Forest,
                        '#' => Biome::City,
                        '^' => Biome::Mountain,
                        _ => panic!("Unknown biome char {}", c),
                    },
                )
            })
            .collect();

        let mut ret = WorldSkeleton::default();
        // Overworld
        for (p, biome) in &map {
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
    pub skeleton: &'a WorldSkeleton,
}

impl<'a> ConnectedSectorSpec<'a> {
    pub fn new(seed: u32, sector: Sector, skeleton: &'a WorldSkeleton) -> Self {
        ConnectedSectorSpec {
            seed,
            sector,
            spec: &skeleton[&sector],
            skeleton,
        }
    }
}

impl<'a> Deref for ConnectedSectorSpec<'a> {
    type Target = SectorSpec;

    fn deref(&self) -> &SectorSpec { self.spec }
}

impl<'a> Distribution<Map> for ConnectedSectorSpec<'a> {
    fn sample(&self, rng: &mut Rng) -> Map {
        match self.biome {
            Biome::Dungeon => self.build_dungeon(rng),
            _ => self.build_biome_sample_map(rng),
        }
    }
}

impl<'a> ConnectedSectorSpec<'a> {
    /// Sector base shape for map generation.
    ///
    /// The base will be deformed if the sector has no neighbors to the north or south. A
    /// standalone sector will be cropped to align with an unscrolling game screen.
    fn base_shape(&self) -> impl Iterator<Item = CellVector> {
        let allow_north = self.neighbor(SectorDir::Northwest).is_some()
            || self.neighbor(SectorDir::Northeast).is_some();
        let allow_south = self.neighbor(SectorDir::Southwest).is_some()
            || self.neighbor(SectorDir::Southeast).is_some();

        Sector::shape().filter(move |p| match SectorPart::from(*p) {
            SectorPart::NorthTriangle => allow_north,
            SectorPart::SouthTriangle => allow_south,
            _ => true,
        })
    }

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

    pub fn neighbor(&self, offset: impl Into<SectorVec>) -> Option<&SectorSpec> {
        self.skeleton.get(&(self.sector + offset.into()))
    }

    fn place_stairs(&self, rng: &mut Rng, map: &mut Map) -> Result<(), Box<dyn Error>> {
        // TODO: Biome affects vault distribution
        if self.neighbor(SectorDir::Up).is_some() {
            let room: Entrance = self.sample(rng);
            debug!("Placing upstairs");
            map.place_room(rng, &*room.0)?;
        }

        if self.neighbor(SectorDir::Down).is_some() {
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
        self.neighbor(SectorDir::Down).map(|_| {
            Location::from(self.sector)
                .v2_at(self.sector.downstairs_location(self.seed))
                .unwrap()
        })
    }

    fn upstairs_pos(&self) -> Option<CellVector> {
        self.neighbor(SectorDir::Up).map(|_| {
            let mut upstairs_pos = (self.sector + vec3(0, 0, 1)).downstairs_location(self.seed);
            upstairs_pos.z -= 1;
            // Offset it so that the exits line up nicer.
            upstairs_pos.x -= 1;
            upstairs_pos.y -= 1;
            Location::from(self.sector).v2_at(upstairs_pos).unwrap()
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
        let mut ret = Map::new_base(Terrain::Rock, self.base_shape());
        self.place_stairwells(&mut ret);
        ret
    }

    fn build_biome_sample_map(&self, rng: &mut Rng) -> Map {
        let mut map = Map::default();
        for p in self.base_shape() {
            let loc = Location::from(self.sector) + p;
            let perturbed_loc = loc + loc.terrain_cell_displacement();
            let mut biome = self.biome;
            // Border noise can make neighboring sector terrain show up on this one.
            let encroaching_sector = Sector::from(perturbed_loc);
            if let Some(sector) = self.skeleton.get(&encroaching_sector) {
                biome = sector.biome;
            }

            // TODO: If biome changes in three neighboring cells, turn terrain to ground
            let terrain = biome.terrain_at(self.seed, loc);

            map.insert(p, MapCell::new_terrain(terrain));
        }

        // TODO: Add enclosures
        self.place_stairwells(&mut map);

        for &pos in &map.open_ground() {
            // TODO: Pick distribution based on biome...
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
    use super::{CellVector, Sector, SECTOR_HEIGHT, SECTOR_HEX_SIDE, SECTOR_WIDTH};
    use calx::{CellSpace, ProjectVec, StaggeredHexSpace};
    use euclid::{vec2, vec3};

    #[test]
    fn test_rect_space() {
        let shape: Vec<CellVector> = Sector::shape().collect();
        for v in 0..SECTOR_HEIGHT {
            for u in 0..SECTOR_WIDTH {
                assert!(shape
                    .iter()
                    .find(|x| x.project::<StaggeredHexSpace>() == vec2(u, v))
                    .is_some());
            }
        }
    }

    #[test]
    fn test_stair_locations() {
        for z in -10..10 {
            let s = Sector::new(0, 0, z * 10);
            let loc = s.downstairs_location(123);

            // Locations must be placed inside sector.
            assert!(s.iter().find(|x| x == &loc).is_some());

            // Location must not collide with the other stair location.
            let mut upstairs_loc = (s + vec3(0, 0, 1)).downstairs_location(123);
            upstairs_loc.z = loc.z;
            assert!(loc.distance_from(upstairs_loc).unwrap() > 1);
        }
    }

    #[test]
    fn test_herringbone_space() {
        use super::HerringboneSpace;

        assert_eq!(
            CellVector::new(5, 5).project::<HerringboneSpace>(),
            vec2(0, 0)
        );
        for y in -10..10 {
            for x in -10..10 {
                let chunk = vec2::<_, HerringboneSpace>(x as i32, y as i32);
                assert_eq!(
                    chunk.project::<CellSpace>().project::<HerringboneSpace>(),
                    chunk
                );
            }
        }
    }

    #[test]
    fn test_sector_shape() {
        assert_eq!(
            Sector::shape().count() as i32,
            3 * SECTOR_HEX_SIDE * SECTOR_HEX_SIDE
        );
    }
}
