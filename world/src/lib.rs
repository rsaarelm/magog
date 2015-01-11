#![crate_name="world"]
#![feature(globs)]
#![feature(macro_rules)]
#![feature(unboxed_closures)]

extern crate rand;
extern crate "rustc-serialize" as rustc_serialize;
extern crate num;
extern crate collect;
extern crate calx;

pub use entity::{Entity};
pub use flags::{camera, set_camera, get_tick};
pub use fov::{Fov};
pub use geom::{HexGeom};
pub use location::{Location, Chart, Unchart};
pub use msg::{pop_msg};
pub use terrain::{TerrainType};
pub use world::{init_world, load, save};
pub use dir6::Dir6;

macro_rules! msg(
    ($($arg:tt)*) => ( ::msg::push_msg(::Msg::Text(format!($($arg)*))))
);

macro_rules! msgln(
    ($($arg:tt)*) => ({
        ::msg::push_msg(::Msg::Text(format!($($arg)*)));
        ::msg::push_msg(::Msg::Text("\n".to_string()));
    })
);

macro_rules! caption(
    ($($arg:tt)*) => ( ::msg::push_msg(::Msg::Caption(format!($($arg)*))))
);

pub mod action;
pub mod components;
pub mod item;

mod ability;
mod area;
mod dir6;
mod ecs;
mod entity;
mod flags;
mod fov;
mod geom;
mod geomorph;
mod geomorph_data;
mod location;
mod mapgen;
mod msg;
mod rng;
mod spatial;
mod stats;
mod terrain;
mod world;

#[derive(Copy, Eq, PartialEq, Show)]
pub enum FovStatus {
    Seen,
    Remembered,
}

/// Landscape type. Also serves as bit field in order to produce habitat masks
/// for entity spawning etc.
#[derive(Copy, Eq, PartialEq, Clone, Show, RustcEncodable, RustcDecodable)]
pub enum Biome {
    Overland = 0b1,
    Dungeon  = 0b10,

    // For things showing up at a biome.
    Anywhere = 0b11111111,
}

impl Biome {
    pub fn default_terrain(self) -> terrain::TerrainType {
        match self {
            Biome::Overland => TerrainType::Tree,
            Biome::Dungeon => TerrainType::Rock,
            _ => TerrainType::Void,
        }
    }
}

#[derive(Copy, Eq, PartialEq, Show, Clone, RustcEncodable, RustcDecodable)]
pub struct AreaSpec {
    pub biome: Biome,
    pub depth: int,
}

impl AreaSpec {
    pub fn new(biome: Biome, depth: int) -> AreaSpec {
        AreaSpec { biome: biome, depth: depth }
    }

    /// Return whether a thing with this spec can be spawned in an environment
    /// with the given spec.
    pub fn can_hatch_in(&self, environment: &AreaSpec) -> bool {
        self.depth >= 0 && self.depth <= environment.depth &&
        (self.biome as int & environment.biome as int) != 0
    }
}

/// Various one-off signals the game sends to the UI layer.
#[derive(Clone, Show)]
pub enum Msg {
    /// Regular event message
    Text(String),
    /// Important event message to the center of the screen
    Caption(String),
    // TODO: Type of effect.
    Explosion(Location),
    Damage(Entity),
    Gib(Location),
}
